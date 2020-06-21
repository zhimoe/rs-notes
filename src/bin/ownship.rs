fn main() {
    let x: &[u8] = &[b'a', b'b', b'c'];
    let stack_str: &str = str::from_utf8(x).unwrap();
    println!("{}", stack_str);
}

fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}

//heap and stack: stack is store data that known,fixed size.
//Keeping track of what parts of code are using what data on the heap,
// minimizing the amount of duplicate data on the heap, and
// cleaning up unused data on the heap so you don’t run out of space are all problems that ownership addresses.
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

    let s3 = s2.clone(); //copy the heap value("hello")
    println!("{}, world!", s2); // s2 still usable
}

// passing function arguments or return value by function is same as assigning a value to a variable, you need take care the ownership of heap value,
fn ownership() {
    let x = 5;
    let x10 = plus10(x);// x still usable since the x is stack data
    println!("{}", x);
    println!("{}", x10);

    let s = String::from("hello");
    takes_ownership(s); //s's value moves into the function and so is no longer valid here
}

fn plus10(i: i32) -> i32 {
    i + 10
}

fn takes_ownership(some_string: String) { // some_string comes into scope
    println!("{}", some_string);
} // Here, some_string goes out of scope and `drop` is called. The backing memory is freed.


//References and Borrowing
//since the ownership is too hard to track by coder's eye, rust introduce the ref and borrowing
// a function that accept a ref will not takeover a value's ownership when the function is called ...
// ... also will not drop the value's backend memory when function is return.
//

//only one mut ref in a scope!!! to prevent data race;
//also &mut ref and &ref can not exist in same scope


//dangling reference
//fn dangle() -> &String {
//    let s = String::from("dangle ref");
////    &s
//}// the s is dropped, but the function try to return s reference


//String vs str vs &String vs &str
//1. String is heap string buffer
//2. &String is a ref of String
//3. str is unknown immutable sequence of utf8 bytes stored somewhere in memory. the memory may be:
//3a. in binary: a string literal "foo" is a &'static str. The data is hardcoded into the executable and loaded into memory when the program runs.
//3b. in heap: Strings implement Deref<Target=str>, and so inherit all of str's methods.
//3c. in stack: when use str::from_utf8(x).unwrap(); x is stack-value ref

//since the str is unknown size, one can only use it by &str, called slice. slice is a view of some data.

fn str() {
    let s = "hello str";//The type of s here is &str: it’s a slice pointing to that specific point of the binary.
    // This is also why string literals are immutable; &str is an immutable reference.
}






