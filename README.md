# Kraber

A fast, powerful, and safe programming language written in rust!

## About

It packs a punch like the Kraber .50-Cal. Kraber is designed to be minimal, flexible, and readable. Everything follows a clear and consistent pattern.

## Syntax

### Basic

- declaring a variable called *foo* with type *integer*: `declare foo as integer`
- setting the variable *foo* to the integer *+1*: `set foo to +1`
- printing *foo*: `foo`

### Scopes

Scoped blocks are defined by curly braces in Kraber. Accessing a local variable outside the block it was declared in will result in an error.

```
declare x as whole
set x to 0
while lt(x 4) {
  declare y as whole
  set x to add(x 1)
  set y to multiply(x 2)
}
y
```

```
called `Option::unwrap()` on a `None` value
```

### Operators

Operators do not exist in Kraber. Instead, everything is done with functions because they are not limited to unary or binary operations. Functions that are both **commutative** and **associative** can take an infinite number of arguments.

### Numeric Types

Kraber only performs type coercion on numeric data types. 

The `+` and `-` signs are not operators but are used to prefix integers (signed) to distinguish them from whole (unsigned) numbers.

## Examples

### Implicit Printing

This makes debugging quick and easy.

```
declare x as text
set x to "Hello, Kraber!"
x
```

### Boolean Operations

Boolean operations are handled by **Kraber Functions** written in rust.

> [!TIP]
> All logic gates can be made using the `nand` gate!

```
declare x as boolean
set x to nand(true true)
x
```

### Arithmetic Operations

```
declare x as float
set x to add(1 2)
set x to multiply(x 2)
set x to raise(x 2)
x
```

## String Operations

```
declare word as text
set word to "OK, "
set word to multiply(word 2)
set word to join(word "thanks for making Kraber!")
word
```

### Loops

> [!TIP]
> While loops can be used to implement `if` statements!

```
declare x as boolean
set x to true
while x
{
  set x to nand(true true)
}
```

### Functions

```
declare factorial as function
set factorial to fun (n as whole) as whole
{
  while equal(n 0) {
    return 1
  }
  return multiply(n factorial(add(n -1)))
}
factorial(69)
```