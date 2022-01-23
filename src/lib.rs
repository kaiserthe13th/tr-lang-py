use pyo3::types::PyBytes;
use pyo3::wrap_pymodule;
use tr_lang::Lexer as TrLexer;
use tr_lang::Parser as TrParser;
use tr_lang::Run as TrRun;
use pyo3::prelude::*;
use tr_lang::token;

#[pyclass]
#[derive(Clone)]
struct Lexer {
    inner: TrLexer,
}

#[pymethods]
impl Lexer {
    #[new]
    fn new(source: String) -> Self {
        Self {
            inner: TrLexer::new(source)
        }
    }
    
    fn tokenize(mut self_: PyRefMut<Self>, file: Option<String>) -> Vec<LexerToken> {
        self_.inner.tokenize(&mut vec![], file.unwrap_or("<python>".to_string()))
            .iter()
            .map(|a| LexerToken::from(a.clone()))
            .collect()
    }
}

#[pyclass]
#[derive(Clone)]
struct Parser {
    inner: TrParser,
}

#[pymethods]
impl Parser {
    #[new]
    fn new(tokens: Vec<LexerToken>) -> Self {
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
        self_.inner.parse()
            .iter()
            .map(|a| ParserToken::from(a.clone()))
            .collect()
    }
}

#[pyclass]
struct Run {
    inner: TrRun,
}

#[pymethods]
impl Run {
    #[new]
    fn new(tokens: Vec<ParserToken>) -> Self {
        Self {
            inner: TrRun::new(
                tokens
                    .iter()
                    .map(|a| -> token::ParserToken { a.clone().into() })
                    .collect()
            )
        }
    }
    fn run(mut self_: PyRefMut<Self>, file: Option<String>) {
        self_.inner.run(file.unwrap_or("<python>".to_string()))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LexerToken {
    inner: token::LexerToken,
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
    fn to_bytes<'a>(py: Python<'a>, tokens: &'a PyBytes) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(py,
            &bytecode::to_bytecode(
                tokens.extract::<Vec<ParserToken>>()?
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
fn tr_lang(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(bytecode))?;

    m.add_class::<LexerToken>()?;
    m.add_class::<Lexer>()?;
    m.add_class::<ParserToken>()?;
    m.add_class::<Parser>()?;
    m.add_class::<Run>()?;

    Ok(())
}
