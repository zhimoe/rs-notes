## packages, crates and modules  
A Cargo.toml is a package. and must have a package name, like 

```toml
[package]
name = "actix-web"
```

A package(project) contains one or more crates;

A package CAN contain as many binary crates as you’d like, but it must contain at least one crate (either library or binary);
use src/main.rs, will build to package-name binary, or use src/bin/b1.rs,src/bin/b2.rs, wil get 2 binaries: b1,b2.

A package must contain zero or one(0或者1个) library crates, and no more.

by convention, package-name is use `-` (dash,can be `_`), but lib_name must use `_` (underscores, can not be `-`);

cargo will auto replace the `-` with `_` in package-name to name the  default library crate(lib.rs in src root). Also you can name it in [lib]:


```toml
[lib]
name = "actix_web"
path = "src/lib.rs"


# also you can rename the binary:
# it use [[]], array in toml
[[bin]]
name = "my-cool-binary"
path = "src/main.rs"

[[bin]]
name = "bin1"
path = "src/bin/bin1.rs"
```

one package(project) can only have one library crate, when the lib continues to get bigger, you want to split up the lib into multiple packages.
cargo introduce you with workspace.

> A workspace is a set of packages that share the same Cargo.lock and output directory.

here is the actix-web package Cargo.toml file:

```toml
[workspace]
members = [
  ".", # this is for current directory src/
  "awc",
  "actix-http",
  "actix-cors",
  "actix-files",
  "actix-framed",
  "actix-session",
  "actix-identity",
  "actix-multipart",
  "actix-web-actors",
  "actix-web-codegen",
  "test-server",
]
# awc,actix-http... all are packages that contains their own Cargo.toml and src/lib.rs; 
``` 

A crate is a compilation unit in Rust. 

Whenever rustc some_file.rs is called, some_file.rs is treated as the crate file. 
If some_file.rs has mod declarations in it, then the contents of the module files would be inserted 
in places where mod declarations in the crate file are found, before running the compiler it. 
In other words, modules do not get compiled individually, only crates get compiled.

`mod mod_name {}` defines a mod.

`mod mod_name; ` cargo will look for mod_name.rs or mod_name/mod.rs and insert the content to current file.

by default the mod is private; but nested mod is allowed to use any code in super mod;

`self` and `super` is to ref the current mod and super mod;

```rust
fn main(){
// absolute path
crate::music::popular::play();
          
// relative path
music::popular::play();
}
```

> the Structs members is all private by default even struct name is pub;

> the Enums members is all public by default if the name is pub; 

## use keyword

the `use` keyword brings path(crate mod path) into scope;

```rust
//bring a module into scope with `use` and a relative path need start `self`:
use self::music::popular;
//!!!the self:: is no needed in rust 2018+

//use the absolute path
use crate::music::popular;

//make the path to public
pub use crate::music::popular;

//use
use std::{cmp::Ordering,io};
use std::{self,Write};
```

## summary

1. 简单粗暴的理解,一个项目 == 一个package, 一个package可以包含多个crate. 
2. crate是Cargo的编译单元,也是Cargo.toml中`[dependencies]`的依赖单元.
3. 一个package只能包含一个lib crate(`src/lib.rs`),但是可以在`src/main.rs`或者`src/bin/*.rs`下面包含任意多个bin crate;
4. 对于复杂项目,可以通过cargo的`[workspace]`管理多个crate,这样可以实现一个`Cargo.toml`管理/构建多个lib crate.

5. mod是rust中代码的组织最小单元. `mod mod_name {}` 是定义一个mod;`mod mod_name; ` 表示将`mod_name.rs`或者`mod_name/mod.rs`中的内容插入到当前文件当前位置,并且插入内容被包含在`mod mod_name`中.
6. crate内部的mod引用使用`self::`开头,引用外部crate则使用`crate::`开头.


## ownership borrowing and lifetimes
```rust
//heap and stack: stack is store data that known,fixed size.
//Keeping track of what parts of code are using what data on the heap, minimizing ...
//the amount of duplicate data on the heap, and cleaning up unused data on the heap ...
//so you don’t run out of space are all problems that ownership addresses.

//ownership rules:
//Each value in Rust has a variable that’s called its owner.
//There can only be one owner at a time.
//When the owner goes out of scope, the value will be dropped.

// stack only data assignment will make a copy operation, since it is fixed size, the copy is fast
// use s.clone make a heap data deeply copy.
// impl the Copy trait can make a type still usable after assignment
// Copy trait can not use with Drop trait
fn copy() {
    let x = 5;
    let y = x; //copy the value(5) in the stack,since it is fixed-size, the copy operation is fast

    let s1 = String::from("hello");
    let s2 = s1; //s1 is invalid
    // println!("{}, world!", s1); //error, the "hello" ownership move to s2

    let s3 = s2.clone(); //copy the heap value("hello"), impl the Clone trait
    println!("{}, world!", s2); // s2 still usable
}

// passing function arguments or return value by function is same as 
// assigning a value to a variable, you need take care the ownership of heap value,
fn ownership() {
    let x = 5;
    let x10 = plus10(x);// x still usable since the x is stack data
    println!("{}", x);
    println!("{}", x10);

    let s = String::from("hello");
    takes_ownership(s); //s's value moves into the function and so is no longer valid here
}

fn plus10(i: i32) -> i32 { // since the i is primitive in stack, so the function return a new value  
    i + 10 
}

fn takes_ownership(some_string: String) { // some_string comes into scope
    println!("{}", some_string);
} // Here, some_string goes out of scope and `drop()` is called. The backing memory is freed.


// References and Borrowing:
// since the ownership is too hard to track by coder's eye, rust introduce the ref and borrowing
// a function that accept a ref will not takeover a value's ownership when the function is called ...
// also will not drop the value's backend memory when function is return.


// a variable can only have one mut ref or many immutable ref in a same scope;

//dangling reference
fn dangle() -> &String {
    let s = String::from("dangle ref");
    &s //error
}// the s is dropped, but the function try to return s reference


//String vs str vs &String vs &str
//1. String is heap string buffer
//2. &String is a ref of String
//3. str is unknown immutable sequence of utf8 bytes stored somewhere in memory. the memory may be:
//  3a. in binary: a string literal "foo" is a &'static str. The data is hardcoded into the executable and loaded into memory when the program runs.
//  3b. in heap: Strings implement Deref<Target=str>, and so inherit all of str's methods.
//  3c. in stack: when use str::from_utf8(x).unwrap(); x is stack-value ref

//> the &str param can accept a &String since the String implement Deref<Target=str>.

//since the str is unknown size, one can only use it by &str, called slice. slice is a view of some data. 


fn str() {
    let s = "hello str";//The type of s here is &str: it’s a slice pointing to that specific point of the binary.
    // This is also why string literals are immutable; &str is an immutable reference.
}

```

## lifetimes are only about reference
```rust
//a ref must die before its referent

//a string slice has static lifetime

//let s: &str = "hello";
//means
//let s: &’static str = "hello";     //



```
in rust: 
- A resource can only have one owner at a time. When it goes out of the scope, Rust removes it from the Memory.

- When we want to reuse the same resource, we are referencing it/ borrowing its content.

- When dealing with references, we have to specify lifetime annotations to provide instructions for the compiler to set how long those referenced resources should be alive.

- ⭐ But because of lifetime annotations make the code more verbose, in order to make common patterns more ergonomic, Rust allows lifetimes to be elided/omitted in fn definitions. In this case, the compiler assigns lifetime annotations implicitly.


```rust
// No inputs, return a reference
fn function1<'a>() -> &'a str {}

// Single input
fn function2<'a>(x: &'a str) {}

// Single input and output, both have the same lifetime
// The output should live at least as long as input exists
fn function3<'a>(x: &'a str) -> &'a str {} // no need the lifetime annotation,lifetime elision

// Multiple inputs, only one input and the output share same lifetime
// The output should live at least as long as y exists
fn function4<'a>(x: i32, y: &'a str) -> &'a str {}

// Multiple inputs, both inputs and the output share same lifetime
// The output should live at least as long as x and y exist
fn function5<'a>(x: &'a str, y: &'a str) -> &'a str {}

// Multiple inputs, inputs can have different lifetimes 🔎
// The output should live at least as long as x exists
fn function6<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {}
```

## lifetimes in struct/enum
```rust
// Single element
// Data of x should live at least as long as Struct exists
struct Struct1<'a> {
    x: &'a str
}

// Multiple elements
// Data of x and y should live at least as long as Struct exists
struct Struct2<'a> {
    x: &'a str,
    y: &'a str
}


// Variant with a single element
// Data of the variant should live at least as long as Enum exists
enum Enum<'a> {
    Variant(&'a Type)
}
```


## 

## feature
