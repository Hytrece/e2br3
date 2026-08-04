use flate2::{write::ZlibEncoder, Compression};
use skrifa::prelude::{LocationRef, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::OnceLock;

const FONT_BYTES: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/../../../assets/fonts/NotoSansCJKkr-Regular.otf"
));

const FONT_NAME: &str = "NotoSansCJKkr-Regular";

pub(super) fn font_name() -> &'static str {
	FONT_NAME
}

pub(super) fn compressed_font() -> &'static [u8] {
	static FONT: OnceLock<Vec<u8>> = OnceLock::new();
	FONT.get_or_init(|| {
		let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
		encoder
			.write_all(cff_table())
			.expect("embedded CIOMS font must be compressible");
		let compressed = encoder
			.finish()
			.expect("embedded CIOMS font compression must finish");
		compressed
	})
	.as_slice()
}

pub(super) fn font_widths() -> &'static str {
	static WIDTHS: OnceLock<String> = OnceLock::new();
	WIDTHS.get_or_init(|| {
		let font = FontRef::new(FONT_BYTES)
			.expect("embedded CIOMS font must be valid OpenType");
		let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
		let glyph_count = font
			.metrics(Size::unscaled(), LocationRef::default())
			.glyph_count as u32;
		let widths = (0..glyph_count)
			.map(|glyph| {
				metrics
					.advance_width(GlyphId::new(glyph))
					.unwrap_or(1000.0)
					.round() as u16
			})
			.collect::<Vec<_>>();
		let mut result = String::from("[");
		let mut start = 0;
		while start < widths.len() {
			let end = (start + 1..widths.len())
				.find(|&index| widths[index] != widths[start])
				.unwrap_or(widths.len());
			let _ = write!(result, " {start} {} {}", end - 1, widths[start]);
			start = end;
		}
		result.push_str(" ]");
		result
	})
}

pub(super) fn glyph_id(codepoint: u32) -> u16 {
	static GLYPHS: OnceLock<BTreeMap<u32, u16>> = OnceLock::new();
	*GLYPHS
		.get_or_init(|| {
			let font = FontRef::new(FONT_BYTES)
				.expect("embedded CIOMS font must be valid OpenType");
			font.charmap()
				.mappings()
				.filter_map(|(codepoint, glyph)| {
					let glyph = glyph.to_u32();
					(codepoint <= 0x10FFFF && glyph <= u16::MAX as u32)
						.then_some((codepoint, glyph as u16))
				})
				.collect()
		})
		.get(&codepoint)
		.unwrap_or(&0)
}

pub(super) fn encoding_cmap(mapping: &[(u16, u32)]) -> String {
	build_cmap(false, mapping)
}

pub(super) fn to_unicode_cmap(mapping: &[(u16, u32)]) -> String {
	build_cmap(true, mapping)
}

fn build_cmap(to_unicode: bool, mapping: &[(u16, u32)]) -> String {
	let name = if to_unicode {
		"NotoSansCJKkr-ToUnicode"
	} else {
		"NotoSansCJKkr-Encoding"
	};
	let cmap_type = if to_unicode { 2 } else { 1 };
	let mut cmap = format!(
		concat!(
			"/CIDInit /ProcSet findresource begin\n",
			"12 dict begin\n",
			"begincmap\n",
			"/CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> def\n",
			"/CMapName /{} def\n",
			"/CMapType {} def\n",
			"1 begincodespacerange\n",
			"<0000> <FFFF>\n",
			"endcodespacerange\n",
		),
		name, cmap_type,
	);

	for chunk in mapping.chunks(100) {
		if to_unicode {
			let _ = writeln!(cmap, "{} beginbfchar", chunk.len());
			for &(code, codepoint) in chunk {
				let _ = writeln!(cmap, "<{code:04X}> <{}>", unicode_hex(codepoint));
			}
			cmap.push_str("endbfchar\n");
		} else {
			let _ = writeln!(cmap, "{} begincidchar", chunk.len());
			for &(code, codepoint) in chunk {
				let _ = writeln!(cmap, "<{code:04X}> {}", glyph_id(codepoint));
			}
			cmap.push_str("endcidchar\n");
		}
	}

	cmap.push_str(concat!(
		"endcmap\n",
		"CMapName currentdict /CMap defineresource pop\n",
		"end\n",
		"end\n",
	));
	cmap
}

fn unicode_hex(codepoint: u32) -> String {
	if codepoint <= 0xFFFF {
		return format!("{codepoint:04X}");
	}
	let value = codepoint - 0x10000;
	let high = 0xD800 + ((value >> 10) as u16);
	let low = 0xDC00 + ((value & 0x3FF) as u16);
	format!("{high:04X}{low:04X}")
}

fn cff_table() -> &'static [u8] {
	let table_count = u16::from_be_bytes([FONT_BYTES[4], FONT_BYTES[5]]) as usize;
	for index in 0..table_count {
		let record = 12 + (index * 16);
		if &FONT_BYTES[record..record + 4] == b"CFF " {
			let offset = u32::from_be_bytes([
				FONT_BYTES[record + 8],
				FONT_BYTES[record + 9],
				FONT_BYTES[record + 10],
				FONT_BYTES[record + 11],
			]) as usize;
			let length = u32::from_be_bytes([
				FONT_BYTES[record + 12],
				FONT_BYTES[record + 13],
				FONT_BYTES[record + 14],
				FONT_BYTES[record + 15],
			]) as usize;
			return &FONT_BYTES[offset..offset + length];
		}
	}
	panic!("embedded CIOMS font has no CFF table")
}
