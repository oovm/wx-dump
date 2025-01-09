use std::fmt::{Debug, Formatter};
use crate::{WxError, WxResult};
use std::str::FromStr;
use sxd_document::Package;
use sxd_xpath::{Context, Factory, Value};

/// A lazy XML parser.
pub struct LazyXML {
    factory: Factory,
    package: Package,
}

impl Debug for LazyXML {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.package, f)
    }
}

impl FromStr for LazyXML {
    type Err = WxError;

    fn from_str(s: &str) -> WxResult<Self> {
        Ok(LazyXML { factory: Factory::new(), package: sxd_document::parser::parse(s)? })
    }
}
impl LazyXML {
    pub fn get_xpath(&self, xpath: &str) -> WxResult<Value> {
        let xpath = match self.factory.build(xpath)? {
            Some(s) => s,
            None => Err(WxError::custom("invalid xpath"))?,
        };
        let context = Context::new();
        Ok(xpath.evaluate(&context, self.package.as_document().root())?)
    }
}
