#[cfg(test)]
mod test;

use {
    std::{
        collections::HashMap,
        fmt::Debug,
    },
    std::path::{ Path, PathBuf, },
};

#[derive(Debug, Default, Clone)]
pub(crate) struct Variable {
    path: PathBuf,
    value: String,
}

impl Variable {
    fn new<P: AsRef<Path>, V: AsRef<str>>(path: P, value: V) -> Self {
        Self { path: path.as_ref().into(), value: value.as_ref().to_owned(), }
    }

    pub(crate) fn value(&self) -> &String {
        &self.value
    }

    pub(crate) fn value_as_path(&self) -> PathBuf {
        let path = PathBuf::from(self.value());

        if path.is_absolute() {
            return path;
        }

        let mut base = self.path.clone();
        base.pop(); // remove dir

        base.push(path);

        base
    }

    pub(crate) fn value_is_truthy<S: AsRef<str>>(value: Option<S>) -> bool {
        !value.is_none_or(|v| matches!(v.as_ref().to_lowercase().as_str(), "n"|"0"|""|"false"))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    args: Vec<String>,
    content: String,
}

impl Function {
    fn new(args: Vec<String>, content: String) -> Self {
        Self { args, content, }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    pub(crate) fn args(&self) -> &Vec<String> {
        &self.args
    }
}

#[derive(Debug, Default, Clone)]
pub struct Context {
    variables: HashMap<String, Vec<Variable>>,
    functions: HashMap<String, Vec<Function>>,
}

impl Context {
    fn variables<K: AsRef<str>>(&self, key: K) -> Option<&Vec<Variable>> {
        self.variables.get(key.as_ref())
    }

    fn variables_mut<K: AsRef<str>>(&mut self, key: K) -> Option<&mut Vec<Variable>> {
        self.variables.get_mut(key.as_ref())
    }

    pub(crate) fn value<K: AsRef<str>>(&self, key: K) -> Option<&String> {
        self.variables(key)?.last().map(|l| l.value())
    }

    pub(crate) fn path<K: AsRef<str>>(&self, key: K) -> Option<PathBuf> {
        let variables = self.variables(key)?;
        Some(variables.last().unwrap().value_as_path())
    }

    pub(crate) fn values<K: AsRef<str>>(&self, key: K) -> Option<Vec<&String>> {
        self.variables(key).map(|variables| variables.iter().map(|s| s.value()).collect::<Vec<&String>>())
    }

    pub fn add_variable<K: AsRef<str>, P: AsRef<Path>, V: AsRef<str>>(&mut self, key: K, path: P, value: V) {
        if self.variables_mut(key.as_ref()).is_none() {
            self.variables.insert(key.as_ref().to_owned(), Vec::new());
        }

        self.variables_mut(key.as_ref()).unwrap().push(Variable::new(path, value));
    }

    pub(crate) fn remove_variable<K: AsRef<str>>(&mut self, key: K) -> Option<Vec<Variable>> {
        if self.variables(key.as_ref()).is_some() {
            return self.variables.remove(key.as_ref());
        }

        None
    }

    pub(crate) fn pop_variable<K: AsRef<str>>(&mut self, key: K) -> Option<Variable> {
        if self.variables(key.as_ref()).is_some() {
            let popped = self.variables_mut(key.as_ref()).unwrap().pop();

            let variables = self.variables(key.as_ref());
            if let Some(variables) = variables && variables.is_empty() {
                self.remove_variable(key.as_ref());
            }

            return popped;
        }

        None
    }

    pub(crate) fn function<K: AsRef<str>>(&self, key: K) -> Option<&Function> {
        self.functions.get(key.as_ref()).and_then(|fns| fns.last())
    }

    pub(crate) fn add_function(&mut self, name: String, args: Vec<String>, content: String) {
        if !self.functions.contains_key(&name) {
            self.functions.insert(name.to_owned(), Vec::new());
        }

        self.functions.get_mut(&name).unwrap().push(Function::new(args, content));
    }
}
