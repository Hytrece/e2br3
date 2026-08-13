use crate::XmlValidationError;
use libxml::xpath::Context;

mod e;
mod g;
mod h;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	e::run(xpath, errors);
	g::run(xpath, errors);
	h::run(xpath, errors);
}
