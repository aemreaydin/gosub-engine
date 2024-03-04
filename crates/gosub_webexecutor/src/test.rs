use crate::js::{
    Args, IntoJSValue, JSContext, JSFunction, JSFunctionCallBack, JSFunctionCallBackVariadic,
    JSFunctionVariadic, JSGetterCallback, JSInterop, JSObject, JSRuntime, JSSetterCallback,
    JSValue, VariadicArgs, VariadicArgsInternal,
};
//use webinterop::{web_fns, web_interop};
use crate::js::v8::V8Engine;
use gosub_shared::types::Result;
use std::cell::RefCell;
use std::rc::Rc;

//#[web_interop]
// struct TestStruct {
//     //#[property]
//     field: i32,
//
//     //#[property]
//     field2: HashMap<i32, i32>, //should crash it
// }
//
// //#[web_fns]
// impl TestStruct {
//     fn add(&self, other: i32) -> i32 {
//         self.field + other
//     }
//
//     fn add2(&mut self, other: i32) {
//         self.field += other
//     }
//
//     fn add3(a: i32, b: i32) -> i32 {
//         a + b
//     }
//     fn variadic<T: VariadicArgsInternal>(_nums: T) {}
//
//     fn v8_variadic(_nums: V8Value) {}
// }

//test, how we need to implement slices and vectors with refs (deref things)
// fn array_test() {
//     let mut test_vec = vec![1, 2, 3];
//
//     vec(test_vec.clone()); //clone only needed for the test
//
//     ref_vec(&test_vec);
//
//     mut_vec(&mut test_vec);
//
//     ref_slice(&test_vec);
//
//     mut_slice(&mut test_vec);
//
//     size_slice(<[i32; 3]>::try_from(test_vec.clone()).unwrap()); //clone only needed for the test
//
//     ref_size_slice(&<[i32; 3]>::try_from(test_vec.clone()).unwrap()); //clone only needed for the test
//
//     mut_size_slice(&mut <[i32; 3]>::try_from(test_vec.clone()).unwrap()); //clone only needed for the test
// }
//
// fn vec(_vec: Vec<i32>) {}
//
// #[allow(clippy::ptr_arg)]
// fn ref_vec(_vec: &Vec<i32>) {}
//
// #[allow(clippy::ptr_arg)]
// fn mut_vec(_vec: &mut Vec<i32>) {}
//
// fn ref_slice(_slice: &[i32]) {}
//
// fn mut_slice(_slice: &mut [i32]) {}
//
// fn size_slice(_array: [i32; 3]) {}
//
// fn ref_size_slice(_slice: &[i32; 3]) {}
//
// fn mut_size_slice(_slice: &mut [i32; 3]) {}

#[derive(Debug)]
struct Test2 {
    field: i32,
    other_field: String,
}

impl Test2 {
    fn cool_fn(&self) -> i32 {
        self.field
    }

    fn add(&mut self, other: i32) {
        self.field += other;
    }

    fn concat(&self, other: String) -> String {
        self.other_field.clone() + &other
    }

    fn takes_ref(&self, other: &String) -> String {
        self.other_field.clone() + other
    }

    fn variadic<A: VariadicArgs>(&self, nums: &A) {
        for a in nums.as_vec() {
            println!("got an arg...: {}", a.as_string().unwrap());
        }
    }
}

impl JSInterop for Test2 {
    //this function will be generated by a macro
    fn implement<RT: JSRuntime>(s: Rc<RefCell<Self>>, mut ctx: RT::Context) -> Result<()> {
        let obj = ctx.new_global_object("test2")?; //#name

        {
            //field getter and setter
            let getter = {
                let s = Rc::clone(&s);
                Box::new(move |cb: &mut RT::GetterCB| {
                    let ctx = cb.context();
                    let value: i32 = s.borrow().field;
                    println!("got a call to getter: {}", value);
                    let value = match value.to_js_value(ctx.clone()) {
                        Ok(value) => value,
                        Err(e) => {
                            cb.error(e);
                            return;
                        }
                    };
                    cb.ret(value);
                })
            };

            let setter = {
                let s = Rc::clone(&s);
                Box::new(move |cb: &mut RT::SetterCB| {
                    let value = cb.value();
                    let value = match value.as_number() {
                        Ok(value) => value,
                        Err(e) => {
                            cb.error(e);
                            return;
                        }
                    };

                    println!("got a call to setter: {}", value);

                    s.borrow_mut().field = value as i32;
                })
            };

            obj.set_property_accessor("field", getter, setter)?;
        }

        {
            //other_field getter and setter
            let getter = {
                let s = Rc::clone(&s);
                Box::new(move |cb: &mut RT::GetterCB| {
                    let ctx = cb.context();
                    let value = s.borrow().other_field.clone();
                    println!("got a call to getter: {}", value);
                    let value = match value.to_js_value(ctx.clone()) {
                        Ok(value) => value,
                        Err(e) => {
                            cb.error(e);
                            return;
                        }
                    };
                    cb.ret(value);
                })
            };

            let setter = {
                let s = Rc::clone(&s);
                Box::new(move |cb: &mut RT::SetterCB| {
                    let value = cb.value();
                    let value = match value.as_string() {
                        Ok(value) => value,
                        Err(e) => {
                            cb.error(e);
                            return;
                        }
                    };

                    println!("got a call to setter: {}", value);

                    s.borrow_mut().other_field = value;
                })
            };

            obj.set_property_accessor("other_field", getter, setter)?;
        }

        let cool_fn = {
            let s = Rc::clone(&s);
            RT::Function::new(ctx.clone(), move |cb| {
                let num_args = 0; //function.arguments.len();
                if num_args != cb.len() {
                    cb.error("wrong number of arguments");
                    return;
                }

                let ctx = cb.context();

                let ret = match s.borrow().cool_fn().to_js_value(ctx.clone()) {
                    Ok(ret) => ret,
                    Err(e) => {
                        cb.error(e);
                        return;
                    }
                };

                cb.ret(ret);
            })?
        };

        obj.set_method("cool_fn", &cool_fn)?;

        let add = {
            let s = Rc::clone(&s);
            RT::Function::new(ctx.clone(), move |cb| {
                let num_args = 1; //function.arguments.len();
                if num_args != cb.len() {
                    cb.error("wrong number of arguments");
                    return;
                }

                let ctx = cb.context();

                let Some(arg0) = cb.args().get(0, ctx.clone()) else {
                    cb.error("failed to get argument");
                    return;
                };

                let Ok(arg0) = arg0.as_number() else {
                    cb.error("failed to convert argument");
                    return;
                };

                #[allow(clippy::unit_arg)]
                let ret = s
                    .borrow_mut()
                    .add(arg0 as i32)
                    .to_js_value(ctx.clone())
                    .unwrap();

                cb.ret(ret);
            })?
        };
        obj.set_method("add", &add)?;

        let concat = {
            let s = Rc::clone(&s);
            RT::Function::new(ctx.clone(), move |cb| {
                let num_args = 1; //function.arguments.len();
                if num_args != cb.len() {
                    cb.error("wrong number of arguments");
                    return;
                }

                let ctx = cb.context();

                let Some(arg0) = cb.args().get(0, ctx.clone()) else {
                    cb.error("failed to get argument");
                    return;
                };

                let Ok(arg0) = arg0.as_string() else {
                    cb.error("failed to convert argument");
                    return;
                };

                let ret = s.borrow().concat(arg0).to_js_value(ctx.clone()).unwrap();

                cb.ret(ret);
            })?
        };
        obj.set_method("concat", &concat)?;

        let takes_ref = {
            let s = Rc::clone(&s);
            RT::Function::new(ctx.clone(), move |cb| {
                let num_args = 1; //function.arguments.len();
                if num_args != cb.len() {
                    cb.error("wrong number of arguments");
                    return;
                }

                let ctx = cb.context();

                let Some(arg0) = cb.args().get(0, ctx.clone()) else {
                    cb.error("failed to get argument");
                    return;
                };

                let Ok(arg0) = arg0.as_string() else {
                    cb.error("failed to convert argument");
                    return;
                };

                let ret = s
                    .borrow()
                    .takes_ref(&arg0)
                    .to_js_value(ctx.clone())
                    .unwrap();

                cb.ret(ret);
            })?
        };
        obj.set_method("takes_ref", &takes_ref)?;

        let variadic = {
            let s = Rc::clone(&s);
            RT::FunctionVariadic::new(ctx.clone(), move |cb| {
                eprintln!("got a call to variadic");
                let ctx = cb.context();

                let args = cb.args().variadic(ctx.clone());

                #[allow(clippy::unit_arg)]
                let ret = s.borrow().variadic(&args).to_js_value(ctx).unwrap();

                cb.ret(ret);
            })?
        };

        obj.set_method_variadic("variadic", &variadic)?;

        Ok(())
    }
}

#[test]
fn manual_js_inop() {
    let mut engine = V8Engine::new();
    let mut context = engine.new_context().unwrap();

    let t2 = Rc::new(RefCell::new(Test2 {
        field: 14,
        other_field: "Hello, ".to_string(),
    }));

    Test2::implement::<V8Engine>(t2.clone(), context.clone()).unwrap();

    let out = context
        .run(
            r#"
        test2.variadic(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
        test2.cool_fn() //  \
        test2.add(3)    //   |-> functions defined in rust
        test2.cool_fn() //  /
        test2.variadic(test2, test2.cool_fn, test2.cool_fn(), test2.field, test2.other_field)

        test2.field += 5
        test2.field = 33
        test2.field
        test2.other_field += "World!"
        test2.other_field
    "#,
        )
        .expect("no value")
        .as_string()
        .unwrap();

    println!("JS: {}", out);
    println!("Rust: {:?}", t2.borrow())
}