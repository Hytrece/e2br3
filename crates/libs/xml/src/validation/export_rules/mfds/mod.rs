use crate::XmlValidationError;
use libxml::xpath::Context;

mod c;
mod d;
mod e;
mod f;
mod g;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	c::run(xpath, errors);
	d::run(xpath, errors);
	e::run(xpath, errors);
	f::run(xpath, errors);
	g::run(xpath, errors);
}
