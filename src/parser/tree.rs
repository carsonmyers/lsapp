use std::fmt;

pub struct Node {
    pub span: Span,
    pub tokens: Vec<Token>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        first = true;

        for tok in self.tokens {
            if !first {
                f.write_char(' ');
            }

            first = false;

            if let Err(_) = f.write_fmt("{:?}", tok) {
                return Err(fmt::Error);
            }
        }

        Ok(())
    }
}

pub struct Section {
    pub node: Node,
    pub heading: String,
    pub entries: Vec<Entry>,
}

impl Debug for Section {
    fn fmt(&self, f: fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt("SECTION({})", self.heading);

        Ok(())
    }
}

pub struct Entry {
    pub node: Node,
    pub key: String,
    pub lang: String,
    pub value: Vec<ValuePart>,

}

impl Debug for Entry {
    fn fmt(&self, f: fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt("ENTRY({} [{}])", self.key, self.lang);

        Ok(())
    }
}

pub enum ValuePart {
    Literal(String),
    Parameter(char),
}

impl Debug for ValuePart {
    fn fmt(&self, f: fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Literal(s) => fmt.write_fmt("\"{}\"", s),
            &Parameter(c) => fmt.write_fmt("%{}", c),
        }

        Ok(())
    }
}

pub enum ValueKind {
    Simple(String),
    Exec(Vec<ValuePart>),
}

impl Debug for ValueKind {
    fn fmt(&self, f: fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Simple(s) => fmt.write_fmt("\"{}\"", s),
            &Exec(ss) => fmt.write_fmt("EXEC({})", ss.join(" ")),
        }
        
        Ok(())
    }
}

pub struct Value {
    pub node: Node,
    pub kind: ValueKind,
}

impl Debug for Value {
    fn fmt(&self, f: fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_fmt("VALUE {:?}", self.kind);
    }
}