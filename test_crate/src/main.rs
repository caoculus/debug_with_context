#![feature(debug_closure_helpers)]

use debug_with_context::{DebugWithContext, DebugWrapContext};
use std::fmt::Debug;

struct Context;
struct GenericContext<T>(T);

#[derive(DebugWithContext)]
#[debug_context(Context)]
struct Test {
    a : i32,
    b : u64,
    c : usize,
    d : String,
}

#[derive(DebugWithContext)]
#[debug_context(Context)]
struct TestGenerics<T, A>
where A: Debug
{
    a : T,
    b : Vec<A>,
}

#[derive(DebugWithContext)]
#[debug_context(Context)]
enum TestE {
    VariantA { a: i32 },
    VariantB { b: i64 },
    VariantC { c: String },
    VariantD { d: Vec<u64> }
}

#[derive(DebugWithContext)]
#[debug_context(Context)]
struct TestT(u64);

#[derive(DebugWithContext)]
#[debug_context(Context)]
struct TestNothing;

#[derive(DebugWithContext)]
#[debug_context(Context)]
enum TestET {
    VariantA(i32),
    VariantB(u64),
}

#[derive(DebugWithContext)]
#[debug_context(Context)]
enum TestEAll {
    VariantA(i32),
    VariantB { b: i64 },
    VariantC,
}

struct Context1;

struct Context2;

#[derive(DebugWithContext)]
struct Test2Context {
    a : i32,
    b : i64,
}

#[derive(DebugWithContext)]
#[debug_context(Context1)]
#[debug_context(Context2)]
struct Test2ContextNoGeneric {
    a : i32,
    b : i64,
}

#[derive(DebugWithContext)]
#[debug_context(Context)]
struct TestAsRefs {
    a : Box<[i64]>,
    b : Vec<i64>,
    c : &'static [i64],
}

#[derive(DebugWithContext)]
#[debug_context(GenericContext<i32>)]
struct TestGenericContextBasic;

#[derive(DebugWithContext)]
#[debug_context(<T> GenericContext<T>)]
struct TestGenericContextTypeParams;

#[derive(DebugWithContext)]
#[debug_context(<T> GenericContext<T> where T: Copy)]
struct TestGenericContextWithWhere;

#[derive(DebugWithContext)]
#[debug_context(<U> GenericContext<U> where U: Clone)]
struct TestContextAndTargetBothWithGenerics<T, A>
where A: Debug
{
    a : T,
    b : Vec<A>,
}


fn main(){

    let context1 = Context1;
    let context2 = Context2;
    let t = Test2Context {
        a: 3,
        b: 6
    };

    println!("{:?}", DebugWrapContext::new(&t, &context1));
    println!("{:?}", DebugWrapContext::new(&t, &context2));

    let context = Context;
    let test = Test {
        a: 3,
        b : 6,
        c : 7,
        d: "oo".to_string(),
    };

    println!("{:?}", DebugWrapContext::new(&test, &context));

    let test = TestNothing;
    println!("{:?}", DebugWrapContext::new(&test, &context));

    let test = TestT(6);
    println!("{:?}", DebugWrapContext::new(&test, &context));

    let teste = TestE::VariantA {a: 2 };
    println!("{:?}", DebugWrapContext::new(&teste, &context));
    let teste = TestE::VariantB { b: 3 };
    println!("{:?}", DebugWrapContext::new(&teste, &context));

    let teste = TestE::VariantA {a: 2 };
    println!("{:?}", DebugWrapContext::new(&teste, &context));
    let teste = TestE::VariantB { b: 3 };
    println!("{:?}", DebugWrapContext::new(&teste, &context));

    let testet = TestET::VariantA(2);
    println!("{:?}", DebugWrapContext::new(&testet, &context));
    let testet = TestET::VariantB(3);
    println!("{:?}", DebugWrapContext::new(&testet, &context));

    let testeboth = TestEAll::VariantA(2);
    println!("{:?}", DebugWrapContext::new(&testeboth, &context));
    let testeboth = TestEAll::VariantB { b: 3 };
    println!("{:?}", DebugWrapContext::new(&testeboth, &context));
    let testeboth = TestEAll::VariantC;
    println!("{:?}", DebugWrapContext::new(&testeboth, &context));
    

    /*let test2contexts = DoubleContext {
        a: 3,
        b: 4,
    };*/

    let testasrefs = TestAsRefs {
        a: Box::new([1, 2]),
        b : vec![3, 4, 5],
        c: &[7, 8, 9],
    };
}
