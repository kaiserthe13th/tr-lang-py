# tr-lang-py
Python bindings for [tr-lang](https://github.com/kaiserthe13th/tr-lang)

## Installation
```console
$ pip install tr-lang-py
```

## Usage
### To Run Inline tr-lang Code
```py
from tr_lang import Lexer, Parser, Run, bytecode

code = """
'Hello, World!\\n' de
"""

lexer = Lexer(code)
parser = Parser(lexer.tokenize())
bytes = bytecode.to_bytes(parser.parse())
run = Run(bytecode.from_bytes(bytes))
run.run() # prints "Hello, World!\n"
```

### To Use tr-lang Files with Python
```py
# /path/to/file.py
from tr_lang import Lexer, Parser, Run

with open("/path/to/tr-lang/file.trl") as f:
    Run(Parser(Lexer(f.read()).tokenize()).parse()).run()
```
```py
# /path/to/tr-lang/file.trl
"What is your name? " de # print("What is your name?", end="")
girdi -> name # name = input()
"Your name is " de name de ".\n" de # print(f"Your name is{name}.")
```

### To Use tr-lang Bytecode from Python
```py
from tr_lang import Run, bytecode

with open("/path/to/bytecode.trl.byt", "rb") as f:
    Run(bytecode.from_bytes(f.read())).run()
```

## Documentation
[tr-lang Documentation](https://tr-lang-docs.netlify.app/english)

## Contributing

To report bugs or suggest new features please use the [issue tracker](https://github.com/kaiserthe13th/tr-lang-py/issues)

Bugfix PR's are welcome!
