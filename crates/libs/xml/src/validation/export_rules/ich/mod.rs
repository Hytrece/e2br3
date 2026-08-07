use crate::XmlValidationError;
use libxml::xpath::Context;

mod c;
mod common;
mod d;
mod e;
mod f;
mod g;
mod h;
mod n;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	n::run(xpath, errors);
	c::run(xpath, errors);
	d::run(xpath, errors);
	e::run(xpath, errors);
	f::run(xpath, errors);
	g::run(xpath, errors);
	h::run(xpath, errors);
}
