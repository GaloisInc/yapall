use std::collections::HashMap;

use regex::RegexSet;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AllocType {
    Heap,
    Stack,
    Top,
}

// TODO: Something for `getline` &co.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Signature {
    // TODO: Implement
    CallsArg { arg: usize },
    ReturnAlloc { r#type: AllocType },
    // ReturnAllocOnce { r#type: AllocType },
    ReturnAliasesArg { arg: usize },
    // ReturnAliasesArgReachable { arg: usize },
    ReturnPointsToGlobal { global: String },
    // ReturnAliasesGlobal { global: String },
    // ReturnAliasesGlobalReachable { global: String },
    // ArgAlloc { arg: usize },
    // ArgAllocOnce { arg: usize },
    ArgMemcpyArg { dst: usize, src: usize },
    // ArgMemcpyArgReachable { dst: usize, src: usize },
    // ArgMemcpyGlobal { dst: usize, global: String },
    // ArgMemcpyGlobalReachable { dst: usize, global: String },
    // ArgPointsToGlobal { arg: usize, global: String },
}

#[derive(Debug)]
pub struct Signatures {
    sigs: HashMap<String, Vec<Signature>>,
    regexes: Vec<String>,
    set: RegexSet,
}

impl Default for Signatures {
    fn default() -> Self {
        Signatures {
            sigs: HashMap::new(),
            regexes: Vec::new(),
            set: RegexSet::new::<[String; 0], _>([]).unwrap(),
        }
    }
}

impl Signatures {
    pub fn new(sigs: HashMap<String, Vec<Signature>>) -> Result<Self, regex::Error> {
        let regexes = sigs.keys().map(|s| s.to_string()).collect();
        let set = RegexSet::new(sigs.keys())?;
        Ok(Signatures { sigs, regexes, set })
    }

    pub fn _has_signatures_for(&self, func: &str) -> bool {
        matches!(self.set.matches(func).into_iter().next(), Some(_))
    }

    pub fn signatures_for(&self, func: &str) -> Option<Vec<Signature>> {
        let mut matched = false;
        let mut sigs = Vec::new();
        for m in self.set.matches(func) {
            matched = true;
            sigs.extend(self.sigs[&self.regexes[m]].clone());
        }
        if matched {
            Some(sigs)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Signatures;

    #[test]
    fn it_works() {
        let sigs = Signatures::new(HashMap::from([]));
        assert_eq!(None, sigs.unwrap().signatures_for("f"));
    }
}
