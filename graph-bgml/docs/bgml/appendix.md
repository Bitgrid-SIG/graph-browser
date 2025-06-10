
### Tag Syntax Forms

* `</tag>` — **Non-void form;** the element is expected to contain inner content and does not
    self-close.
* `<tag/>` — **Void form;** the element must not contain inner content and is self-closing.
* `</tag/>` — **Flexible form;** the element may contain inner content. If none is present,
    it __must__ self-close.
* `<tag>` — **Malformed;** not a valid form within this appendix.

---

### Attribute and Content Notation

* `attr` — **Boolean attribute;** a presence-only attribute without an assigned value.
* `attr=` — **Key-value attribute;** specifies a name-value pair, such as `attr="..."`.
* `.inner` — **Inner content requirement;** denotes that the element must contain inner content
    to be valid.
* `attr?` — **Optional expression;** the attribute or grouped expression immediately preceding
    the `?` is optional and may be omitted.
* `attr | attr` — **Exclusive-or expression;** at least one of the attributes must be present.
* `attr & attr` — **Conjunctive expression;** both attributes are required.
* `attr ^ attr` — **Mutually exclusive expression;** only one of the attributes may be present.
* `(attr ... attr)` — **Precedence grouping;** enforces higher precedence for grouped attributes
    within an expression, overriding default precedence rules at the same syntactic level.
* `attr: ...` — **Attribute definition;** values assigned to this attribute must conform to
    the following expression.
* `attr?: ...` — **Attribute default;** if no value is assigned to this attribute, it will
    default to the following value ; only applicable for optional attributes

---

### Tags & Attributes

- `</meta>`
- `</prelude>`
- `</content>`
- `</postlude>`
- `</maplet/>`
    - (key= & value=) | (key= & .inner) | (value= & .inner)
    - key?: .inner
    - value?: .inner
- `</resource/>`
    - mime= ( \<mimetype> )
    - src= ( \<location> )
    - name= ( \<string> )
    - defer
- `</script>`
    - lang=
    - kind=
    - defer
    - lang: ( "lua" | "teal" | "javascript" | "typescript" )
    - kind: ( "script" | "module" )?
- `</label/>`
    - value=
    - .inner
- `</textinput/>`
    - kind= + validator=?
    - kind= ( "area" | "email" | "password" | "phone" | "date" | "time" | "datetime" )
    - validator=
- `</numberinput/>`
    - kind=
- `</...input/>`
- `<>`
- `<>`
- `<>`



