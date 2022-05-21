// Helper class, used by [[MarkdownIt#core]], [[MarkdownIt#block]] and
// [[MarkdownIt#inline]] to manage sequences of functions (rules):
//
// - keep rules in defined order
// - assign the name to each rule
// - enable/disable rules
// - add/replace rules
// - allow assign rules to additional named chains (in the same)
// - cacheing lists of active rules
//
// You will not need use this class directly until write plugins. For simple
// rules control use [[MarkdownIt.disable]], [[MarkdownIt.enable]] and
// [[MarkdownIt.use]].
//
use derivative::Derivative;
use std::collections::HashMap;

pub struct RuleItem<Rule> {
    //name: &'static str,
    enabled: bool,
    func: Rule,
    alt: Vec<&'static str>,
}

impl<Rule> RuleItem<Rule> {
    pub fn alt(&mut self, chains: Vec<&'static str>) {
        for chain in chains {
            self.alt.push(chain);
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Ruler<Rule> {
    #[derivative(Debug="ignore")]
    __rules__: Vec<RuleItem<Rule>>,

    // Cached rule chains.
    __cache__: Option<HashMap<&'static str, Vec<usize>>>,
}

impl<Rule> Ruler<Rule> {
    pub fn new() -> Self {
        Self {
            __rules__: Vec::new(),
            __cache__: None,
        }
    }

    // Find rule index by name
    //
    /*fn __find__(&self, name: &str) -> Option<usize> {
        for (i, rule) in self.__rules__.iter().enumerate() {
            if rule.name == name {
                return Some(i);
            }
        }
        None
    }*/

    // Build rules lookup cache
    //
    pub fn compile(&mut self) {
        let mut cache = HashMap::new();
        let mut chains = Vec::new();
        chains.push(&"");

        // collect unique names
        for rule in self.__rules__.iter() {
            if !rule.enabled { continue; }

            for alt_name in &rule.alt {
                if !chains.contains(&alt_name) {
                    chains.push(&alt_name);
                }
            }
        }

        for chain in chains {
            let mut vec = Vec::new();

            for (idx, rule) in self.__rules__.iter().enumerate() {
                if !rule.enabled { continue; }

                if !chain.is_empty() && !rule.alt.contains(chain) { continue; }

                vec.push(idx);
            }

            cache.insert(*chain, vec);
        }

        self.__cache__ = Some(cache);
    }

    pub fn push(&mut self, _rule_name: &'static str, func: Rule) -> &mut RuleItem<Rule> {
        self.__rules__.push(RuleItem {
            //name: rule_name,
            enabled: true,
            func: func,
            alt: Vec::new(),
        });

        self.__cache__ = None;

        self.__rules__.last_mut().unwrap()
    }

    // Return array of active functions (rules) for given chain name. It analyzes
    // rules configuration, compiles caches if not exists and returns result.
    //
    pub fn get_rules(&self, chain_name: &str) -> impl Iterator<Item = &Rule> + '_ {
        static NULLVEC : Vec<usize> = Vec::new();

        let iter = if let Some(vec) = self.__cache__.as_ref().unwrap().get(chain_name) {
            vec.iter()
        } else {
            NULLVEC.iter()
        };

        iter.map(|i| &self.__rules__.get(*i).unwrap().func)
    }
}
