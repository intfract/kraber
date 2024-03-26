# Kraber

A fast, powerful, and safe programming language written in rust!

## About

It packs a punch like the Kraber .50-Cal.

## Syntax

- declaring a variable: `declare <identifier> as <type>`
- setting a variable `set <identifier> to <expression>`
- printing `<expression>`

## Examples

### Implicit Printing

This makes debugging quick and easy.

```
declare x as text
set x to "Hello, Kraber!"
x
```

### Boolean Operations

Boolean operations are handled by **kraber functions** written in rust.

> [!TIP]
> All logic gates can be made using the `nand` gate!

```
declare x as boolean
set x to nand(true true)
```

## Arithmetic Operations

```
declare x as float
set x to add(1 2)
set x to multiply(x 2)
set x to raise(x 2)
x
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
  declare result as whole
  set result to 1
  declare counter as whole
  set counter to 2
  while nand(equal(n counter), equal(n counter)) {
    set result to multiply(result counter)
    set counter to add(counter 1)
  }
  set result to multiply(result counter)
  return result
}
declare x as whole
set x to factorial(4)
```