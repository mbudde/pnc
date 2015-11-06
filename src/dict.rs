
use std::rc::Rc;
use std::collections::HashMap;
use words::{Word, BuiltinWord, Operation, block};

#[derive(Debug, PartialEq, Eq)]
pub enum Entry {
    Alias(Word),
    Op(Rc<Operation>),
}

pub struct Dictionary {
    map: HashMap<Word, Entry>,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary { map: HashMap::new() }
    }

    pub fn insert<T>(&mut self, word: T, op: Operation)
        where String: From<T>
    {
        self.map.insert(From::from(word), Entry::Op(Rc::new(op)));
    }

    pub fn insert_alias<T>(&mut self, word: T, other: T)
        where String: From<T>
    {
        self.map.insert(From::from(word), Entry::Alias(From::from(other)));
    }

    pub fn lookup(&self, word: &str) -> Option<Rc<Operation>> {
        let mut entry = self.map.get(word);
        while let Some(&Entry::Alias(ref word)) = entry {
            entry = self.map.get(word);
        }
        match entry {
            Some(&Entry::Op(ref op)) => Some(op.clone()),
            _ => None,
        }
    }
}

impl Default for Dictionary {
    fn default() -> Dictionary {
        let mut dict = Dictionary::new();
        dict.insert("add", Operation::Builtin(BuiltinWord::Add));
        dict.insert("sub", Operation::Builtin(BuiltinWord::Sub));
        dict.insert("mul", Operation::Builtin(BuiltinWord::Mul));
        dict.insert("div", Operation::Builtin(BuiltinWord::Div));
        dict.insert("print", Operation::Builtin(BuiltinWord::Print));
        dict.insert("pop", Operation::Builtin(BuiltinWord::Pop));
        dict.insert("dup", Operation::Builtin(BuiltinWord::Duplicate));
        dict.insert("stdin", Operation::Builtin(BuiltinWord::Stdin));
        dict.insert("sum", Operation::Builtin(BuiltinWord::Sum));
        dict.insert("len", Operation::Builtin(BuiltinWord::Length));
        dict.insert("swap", Operation::Builtin(BuiltinWord::Swap));
        dict.insert("def", Operation::Builtin(BuiltinWord::Def));
        dict.insert("alias", Operation::Builtin(BuiltinWord::Alias));
        dict
    }
}

#[cfg(test)]
mod tests {
    use {BuiltinWord, Operation};
    use super::*;

    #[test]
    fn test() {
        let mut dict: Dictionary = Default::default();
        assert_eq!(dict.lookup("add"),
                   Some(&Operation::Builtin(BuiltinWord::Add)));
        assert_eq!(dict.lookup("+"),
                   Some(&Operation::Builtin(BuiltinWord::Add)));
        assert_eq!(dict.lookup("plus"), None);
        dict.insert_alias("plus".to_string(), "+".to_string());
        assert_eq!(dict.lookup("plus"),
                   Some(&Operation::Builtin(BuiltinWord::Add)));
        dict.insert("incr".to_string(),
                    Operation::Block(vec!["1", "+"].into_iter().map(|s| s.to_string()).collect()));
        assert_eq!(dict.lookup("incr"),
                   Some(&Operation::Block(vec!["1", "+"]
                                              .into_iter()
                                              .map(|s| s.to_string())
                                              .collect())));
    }
}
