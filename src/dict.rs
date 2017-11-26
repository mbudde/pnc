
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use words::{Word, BuiltinWord, Operation};

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Alias(Word),
    Op(Rc<Operation>),
}

struct Inner {
    map: HashMap<Word, Entry>,
    parent: Option<Rc<RefCell<Inner>>>,
}

impl Inner {
    fn lookup(&self, word: &str) -> Option<Rc<Operation>> {
        let mut entry = self.map.get(word);
        while let Some(&Entry::Alias(ref word)) = entry {
            match self.map.get(word) {
                Some(ent) => {
                    entry = Some(ent);
                }
                None => {
                    return self.parent.as_ref().and_then(|p| p.borrow().lookup(word));
                }
            }
        }
        match entry {
            Some(&Entry::Op(ref op)) => Some(Rc::clone(op)),
            _ => {
                self.parent.as_ref().and_then(|p| p.borrow().lookup(word))
            }
        }
    }

    fn available_words(&self) -> BTreeMap<Word, Vec<Word>> {
        let mut words = BTreeMap::new();
        for (word, entry) in &self.map {
            match *entry {
                Entry::Alias(ref alias) => {
                    words.entry(alias.clone()).or_insert_with(|| vec![])
                        .push(word.clone());
                }
                _ => {
                    words.entry(word.clone()).or_insert_with(|| vec![]);
                }
            }
        }
        if let Some(ref parent) = self.parent {
            for (word, aliases) in parent.borrow().available_words() {
                use std::collections::btree_map::Entry::*;
                match words.entry(word) {
                    Vacant(entry) => {
                        entry.insert(aliases);
                    }
                    Occupied(mut entry) => {
                        entry.get_mut().extend(aliases);
                    }
                }
            }
        }
        words
    }
}

pub struct Dictionary {
    inner: Rc<RefCell<Inner>>,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary {
            inner: Rc::new(RefCell::new(Inner {
                map: HashMap::new(),
                parent: None,
            })),
        }
    }

    pub fn with_parent(dict: &Dictionary) -> Dictionary {
        Dictionary {
            inner: Rc::new(RefCell::new(Inner {
                map: HashMap::new(),
                parent: Some(Rc::clone(&dict.inner)),
            })),
        }
    }

    pub fn insert<T>(&mut self, word: T, op: Operation)
        where String: From<T>
    {
        self.inner.borrow_mut().map.insert(From::from(word), Entry::Op(Rc::new(op)));
    }

    pub fn insert_alias<T>(&mut self, word: T, other: T)
        where String: From<T>
    {
        self.inner.borrow_mut().map.insert(From::from(word), Entry::Alias(From::from(other)));
    }

    pub fn lookup(&self, word: &str) -> Option<Rc<Operation>> {
        self.inner.borrow().lookup(word)
    }

    pub fn available_words(&self) -> BTreeMap<Word, Vec<Word>> {
        self.inner.borrow().available_words()
    }
}

impl Default for Dictionary {
    fn default() -> Dictionary {
        let mut dict = Dictionary::new();
        dict.insert("apply", Operation::Builtin(BuiltinWord::Apply));
        dict.insert("add", Operation::Builtin(BuiltinWord::Add));
        dict.insert("alias", Operation::Builtin(BuiltinWord::Alias));
        dict.insert("def", Operation::Builtin(BuiltinWord::Def));
        dict.insert("div", Operation::Builtin(BuiltinWord::Div));
        dict.insert("dup", Operation::Builtin(BuiltinWord::Duplicate));
        dict.insert("fold", Operation::Builtin(BuiltinWord::Fold));
        dict.insert("fold1", Operation::Builtin(BuiltinWord::Fold1));
        dict.insert("len", Operation::Builtin(BuiltinWord::Length));
        dict.insert("map", Operation::Builtin(BuiltinWord::Map));
        dict.insert("filter", Operation::Builtin(BuiltinWord::Filter));
        dict.insert("mul", Operation::Builtin(BuiltinWord::Mul));
        dict.insert("over", Operation::Builtin(BuiltinWord::Over));
        dict.insert("pop", Operation::Builtin(BuiltinWord::Pop));
        dict.insert("print", Operation::Builtin(BuiltinWord::Print));
        dict.insert("dump", Operation::Builtin(BuiltinWord::Dump));
        dict.insert("repeat", Operation::Builtin(BuiltinWord::Repeat));
        dict.insert("roll3", Operation::Builtin(BuiltinWord::Roll3));
        dict.insert("stdin", Operation::Builtin(BuiltinWord::Stdin));
        dict.insert("sub", Operation::Builtin(BuiltinWord::Sub));
        dict.insert("swap", Operation::Builtin(BuiltinWord::Swap));
        dict.insert("arg", Operation::Builtin(BuiltinWord::Arg));

        dict.insert("min", Operation::Builtin(BuiltinWord::Min));
        dict.insert("max", Operation::Builtin(BuiltinWord::Max));

        dict.insert("cmp", Operation::Builtin(BuiltinWord::Cmp));
        dict.insert("if", Operation::Builtin(BuiltinWord::If));
        dict
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use words::{BuiltinWord, Operation};
    use super::*;

    #[test]
    fn test() {
        let mut dict: Dictionary = Default::default();
        assert_eq!(dict.lookup("add"),
                   Some(Rc::new(Operation::Builtin(BuiltinWord::Add))));
        assert_eq!(dict.lookup("plus"), None);
        dict.insert_alias("plus", "add");
        assert_eq!(dict.lookup("plus"),
                   Some(Rc::new(Operation::Builtin(BuiltinWord::Add))));
        dict.insert("incr",
                    Operation::Block(vec!["1", "+"].into_iter().map(|s| s.to_owned()).collect()));
        assert_eq!(dict.lookup("incr"),
                   Some(Rc::new(Operation::Block(vec!["1", "+"]
                                                     .into_iter()
                                                     .map(|s| s.to_string())
                                                     .collect()))));
    }

    #[test]
    fn test_parent() {
        let mut dict: Dictionary = Default::default();
        dict.insert_alias("plus", "add");
        assert_eq!(dict.lookup("plus"),
                   Some(Rc::new(Operation::Builtin(BuiltinWord::Add))));

        let mut sub = Dictionary::with_parent(&dict);
        assert_eq!(sub.lookup("plus"),
                   Some(Rc::new(Operation::Builtin(BuiltinWord::Add))));

        sub.insert_alias("+", "plus");
        assert_eq!(sub.lookup("+"),
                   Some(Rc::new(Operation::Builtin(BuiltinWord::Add))));
    }
}
