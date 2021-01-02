use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::http::RawStr;
use std::borrow::Cow;
use std::fmt;

use rocket::request::FromParam;
use std::iter;

pub struct PasteID<'a>(Cow<'a, str>);

impl<'a> PasteID<'a> {
    pub fn generate() -> PasteID<'static> {
        let mut rng = thread_rng();
        let rand_string: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(10)
            .collect();
        PasteID(Cow::Owned(rand_string))
    }
}

impl<'a> fmt::Display for PasteID<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn valid_id(id: &str) -> bool {
    if id.len() == 10 {
        id.chars().all(char::is_alphanumeric)
    } else {
        false
    }
}

impl<'a> FromParam<'a> for PasteID<'a> {
    type Error = &'a RawStr;

    fn from_param(param: &'a RawStr) -> Result<PasteID<'a>, &'a RawStr> {
        match valid_id(param) {
            true => Ok(PasteID(Cow::Borrowed(param))),
            false => Err(param),
        }
    }
}
