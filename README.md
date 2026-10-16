Translate syntax similar to abnf into lalrpop and use lalrpop to verify grammar correctness

This allows us to focus on constructing syntax rather than syntax+implementation

Examples
-------------------------------------------------------------------------------
```abnf
string = "#string"
ident  = r"[a-zA-Z0-9_\-]+"
name   = r"[a-zA-Z0-9_\-]+\s+="
atom = string / ident
pat  = atom
     / "*" pat
     / "+" pat
     / "(" [pats] ")"
     / "["  pats  "]"
pats = [pats] ident
def  = name pats *("/" pats) [";"]
defs = ["exports" "[" +name "]"]
       ["inlines" "[" +name "]"]
       *def
```

`abnf2lalrpop` outputs:

```lalrpop
grammar;
pub string: () = {
    "#string" => (),
}
pub ident: () = {
    r"[a-zA-Z0-9_\-]+" => (),
}
pub name: () = {
    r"[a-zA-Z0-9_\-]+\s+=" => (),
}
pub atom: () = {
    string => (),
    ident => (),
}
pub pat: () = {
    atom => (),
    "*" pat => (),
    "+" pat => (),
    "(" pats? ")" => (),
    "[" pats "]" => (),
}
pub pats: () = {
    pats? ident => (),
}
pub def: () = {
    name pats ("/" pats)* ";"? => (),
}
pub defs: () = {
    ("exports" "[" name+ "]")? ("inlines" "[" name+ "]")? def* => (),
}
```
