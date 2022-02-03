use std::collections::HashMap;

use pyo3::basic::PyObjectProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyBytes;
use pyo3::exceptions::PyException;
use pyo3::create_exception;
use pyo3::wrap_pymodule;
use tr_lang::Lexer as TrLexer;
use tr_lang::Parser as TrParser;
use tr_lang::Run as TrRun;
use pyo3::prelude::*;
use tr_lang::token;
use tr_lang::mem;
use pyo3::{IntoPy, FromPyObject};

#[derive(FromPyObject)]
enum TrlObject {
    Number(f64),
    String(String),
    Bool(bool),
}
impl IntoPy<PyObject> for TrlObject {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::Number(n) => n.into_py(py),
            Self::String(s) => s.into_py(py),
            Self::Bool(b)   => b.into_py(py),
        }
    }
}
impl Into<mem::Object> for TrlObject {
    fn into(self) -> mem::Object {
        match self {
            Self::String(s) => mem::Object::Yazı(s),
            Self::Number(n) => mem::Object::Sayı(n),
            Self::Bool(b)   => mem::Object::Bool(b),
        }
    }
}
impl TryFrom<mem::Object> for TrlObject {
    type Error = ();
    fn try_from(value: mem::Object) -> Result<Self, ()> {
        match value {
            mem::Object::Bool(b) => Ok(TrlObject::Bool(b)),
            mem::Object::Sayı(n) => Ok(TrlObject::Number(n)),
            mem::Object::Yazı(s) => Ok(TrlObject::String(s)),
            _ => Err(())
        }
    }
}

create_exception!(tr_lang, TrlError, PyException);

#[pyclass]
struct Memory {
    #[pyo3(get, set)]
    pub stack: StackMemory,
    #[pyo3(get, set)]
    pub hashs: HashMemory,
}
#[pymethods]
impl Memory {
    #[new]
    fn new(stack: StackMemory, hashs: HashMemory) -> Self {
        Self { stack, hashs }
    }
}

#[derive(Clone)]
#[pyclass]
struct HashMemory {
    inner: mem::HashMemory,
}

impl From<mem::HashMemory> for HashMemory {
    fn from(inner: mem::HashMemory) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl HashMemory {
    #[new]
    fn new() -> Self {
        Self { inner: mem::HashMemory::new() }
    }
    fn insert(&mut self, key: String, object: TrlObject) {
        self.inner.insert(key, object.into())
    }
    fn insert_glob(&mut self, key: String, object: TrlObject) {
        self.inner.insert_glob(key, object.into());
    }
    fn remove(&mut self, key: String) -> PyResult<Option<TrlObject>> {
        match self.inner.remove(key) {
            Some(o) => Ok(Some(match o {
                mem::Object::Bool(b) => TrlObject::Bool(b),
                mem::Object::Yazı(s) => TrlObject::String(s),
                mem::Object::Sayı(n) => TrlObject::Number(n),
                _ => return Err(PyValueError::new_err("couldn't convert to TrlObject: unsupported type"))
            })),
            None => Ok(None),
        }
    }
    fn new_hash(&mut self) {
        self.inner.new_hash()
    }
    fn del_hash(&mut self) -> PyResult<Option<HashMap<String, TrlObject>>> {
        let r = self.inner.del_hash();
        match r {
            Some(r) => {
                let mut ret = HashMap::new();
                for (k, v) in r.into_iter() {
                    match TrlObject::try_from(v) {
                        Ok(o) => ret.insert(k, o),
                        Err(_) => return Err(PyValueError::new_err("couldn't convert to TrlObject: unsupported type")),
                    };
                }
                Ok(Some(ret))
            },
            _ => Ok(None)
        }
    }
}

#[derive(Clone)]
#[pyclass]
struct StackMemory {
    inner: mem::StackMemory,
}

impl From<mem::StackMemory> for StackMemory {
    fn from(inner: mem::StackMemory) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl StackMemory {
    #[new]
    fn new() -> Self {
        Self { inner: mem::StackMemory::new() }
    }
    fn push(&mut self, o: TrlObject) {
        self.inner.push(o.into());
    }
    fn push_ret(&mut self, o: TrlObject) {
        self.inner.push_ret(o.into());
    }
    fn push_glob(&mut self, o: TrlObject) {
        self.inner.push_glob(o.into());
    }
    fn pop(&mut self) -> PyResult<TrlObject> {
        match self.inner.pop() {
            Some(o) => match o {
                mem::Object::Bool(b) => Ok(TrlObject::Bool(b)),
                mem::Object::Yazı(s) => Ok(TrlObject::String(s)),
                mem::Object::Sayı(n) => Ok(TrlObject::Number(n)),
                _ => Err(PyValueError::new_err("couldn't convert to TrlObject: unsupported type"))
            }
            None => Err(PyValueError::new_err("None")),
        }
    }
    fn new_stack(&mut self) {
        self.inner.new_stack()
    }
    fn del_stack(&mut self) -> PyResult<Option<Vec<TrlObject>>> {
        match self.inner.del_stack() {
            Some(v) => {
                let mut r = v.into_iter().map(|o| match o {
                    mem::Object::Bool(b) => Ok(TrlObject::Bool(b)),
                    mem::Object::Sayı(n) => Ok(TrlObject::Number(n)),
                    mem::Object::Yazı(s) => Ok(TrlObject::String(s)),
                    _ => Err(()),
                });
                if r.all(|a| a.is_ok()) {
                    Ok(Some(r.map(|a| a.unwrap()).collect()))
                } else {
                    Err(PyValueError::new_err("couldn't convert to TrlObject: unsupported type"))
                }
            }
            None => Ok(None),
        }
    }
}

/// Lexer is used for tokenization of strings
/// > It can also load other files in it
#[pyclass]
#[derive(Clone)]
struct Lexer {
    inner: TrLexer,
}

#[pymethods]
impl Lexer {
    #[new]
    fn new(source: String) -> Self {
        //! Create new lexer parsing *source*
        Self {
            inner: TrLexer::new(source)
        }
    }

    fn tokenize(mut self_: PyRefMut<Self>, file: Option<String>) -> Vec<LexerToken> {
        //! Tokenize the tr-lang source code
        self_.inner.tokenize(&mut vec![], file.unwrap_or("<python>".to_string()))
            .iter()
            .map(|a| LexerToken::from(a.clone()))
            .collect()
    }
}

/// Takes in tokens generated by a Lexer
/// and turns them into `ParserToken`s
#[pyclass]
#[derive(Clone)]
struct Parser {
    inner: TrParser,
}

#[pymethods]
impl Parser {
    #[new]
    fn new(tokens: Vec<LexerToken>) -> Self {
        //! Generate new parser using tokens generated by a Lexer
        Self {
            inner: TrParser::new(
                tokens
                    .iter()
                    .map(|a| -> token::LexerToken { a.clone().into() })
                    .collect()
            )
        }
    }

    fn parse(mut self_: PyRefMut<Self>) -> Vec<ParserToken> {
        //! Parse contents, consuming self
        self_.inner.parse()
            .iter()
            .map(|a| ParserToken::from(a.clone()))
            .collect()
    }
}

/// tr-lang runtime
#[pyclass]
struct Run {
    inner: TrRun,
}

#[pymethods]
impl Run {
    #[new]
    fn new(tokens: Vec<ParserToken>) -> Self {
        //! Initialize new runtime
        Self {
            inner: TrRun::new(
                tokens
                    .iter()
                    .map(|a| -> token::ParserToken { a.clone().into() })
                    .collect()
            )
        }
    }

    fn run(mut self_: PyRefMut<Self>, file: Option<String>) -> PyResult<Memory> {
        //! Run contents
        match self_.inner.run(file.unwrap_or("<python>".to_string()), None, false) {
            Ok((s, h)) => Ok(Memory::new(StackMemory::from(s), HashMemory::from(h))),
            Err((_, _, e)) => Err(TrlError::new_err(format!("{:?}", e))),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LexerToken {
    inner: token::LexerToken,
}

#[pyproto]
impl <'a>PyObjectProtocol<'a> for LexerToken {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }
}

impl Into<token::LexerToken> for LexerToken {
    fn into(self) -> token::LexerToken {
        self.inner
    }
}

impl From<token::LexerToken> for LexerToken {
    fn from(inner: token::LexerToken) -> Self {
        Self { inner }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ParserToken {
    inner: token::ParserToken,
}

#[pyproto]
impl <'a>PyObjectProtocol<'a> for ParserToken {
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner))
    }
}

impl Into<token::ParserToken> for ParserToken {
    fn into(self) -> token::ParserToken {
        self.inner
    }
}

impl From<token::ParserToken> for ParserToken {
    fn from(inner: token::ParserToken) -> Self {
        Self { inner }
    }
}

impl ParserToken {
    pub fn inner(&self) -> token::ParserToken {
        self.inner.clone()
    }
}

#[pymodule]
fn bytecode(_py: Python, m: &PyModule) -> PyResult<()> {
    use tr_lang::bytecode;

    #[pyfunction]
    fn to_bytes<'a>(py: Python<'a>, tokens: Vec<ParserToken>) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(py,
            &bytecode::to_bytecode(
                tokens
                    .iter()
                    .map(|a| a.inner())
                    .collect()
            )
        ))
    }

    #[pyfunction]
    fn from_bytes<'a>(bytes: &'a PyBytes) -> PyResult<Vec<ParserToken>> {
        Ok(
            bytecode::from_bytecode(bytes.extract()?)
                .iter()
                .map(|a| ParserToken::from(a.clone()))
                .collect()
        )
    }

    m.add_function(wrap_pyfunction!(to_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(from_bytes, m)?)?;

    Ok(())
}

#[pymodule]
fn tr_lang(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(bytecode))?;

    m.add_class::<LexerToken>()?;
    m.add_class::<Lexer>()?;
    m.add_class::<ParserToken>()?;
    m.add_class::<Parser>()?;
    m.add_class::<Run>()?;
    m.add_class::<Memory>()?;
    m.add_class::<StackMemory>()?;
    m.add_class::<HashMemory>()?;

    m.add("TrlError", py.get_type::<TrlError>())?;

    Ok(())
}
