use std::marker::PhantomData;

use tao_macros::android_fn;

struct JNIEnv<'a> {
  _marker: &'a PhantomData<()>,
}
#[repr(C)]
struct JClass<'a> {
  _marker: &'a PhantomData<()>,
}

android_fn![com_example, tao_app, SomeClass, add, []];
unsafe fn add(_env: JNIEnv, _class: JClass) {}

android_fn![com_example, tao_app, SomeClass, add2, [i32, i32]];
unsafe fn add2(_env: JNIEnv, _class: JClass, _a: i32, _b: i32) {}

android_fn![com_example, tao_app, SomeClass, add3, [i32, i32], i32];
unsafe fn add3(_env: JNIEnv, _class: JClass, a: i32, b: i32) -> i32 {
  a + b
}

android_fn![com_example, tao_app, SomeClass, add4, [], i32];
unsafe fn add4(_env: JNIEnv, _class: JClass) -> i32 {
  0
}

android_fn![com_example, tao_app, SomeClass, add5, [], __VOID__];
unsafe fn add5(_env: JNIEnv, _class: JClass) {}

android_fn![com_example, tao_app, SomeClass, add6, [i32], __VOID__];
unsafe fn add6(_env: JNIEnv, _class: JClass, _a: i32) {}

fn __setup__() {}
fn __store_package_name__() {}
android_fn!(
  com_example,
  tao_app,
  SomeClass,
  add7,
  [i32, i32],
  __VOID__,
  [__setup__, main],
  __store_package_name__,
);
unsafe fn add7(_env: JNIEnv, _class: JClass, _a: i32, _b: i32, _setup: fn(), _main: fn()) {}

android_fn!(
  com_example,
  tao_app,
  SomeClass,
  add8,
  [i32, i32],
  i32,
  [],
  __store_package_name__,
);
unsafe fn add8(_env: JNIEnv, _class: JClass, _a: i32, _b: i32) -> i32 {
  0
}

android_fn![
  com_example,
  tao_app,
  SomeClass,
  add10,
  [JClass<'local>, i32],
  JClass<'local>
];
unsafe fn add10<'local>(
  _env: JNIEnv<'local>,
  _class: JClass<'local>,
  a: JClass<'local>,
  _b: i32,
) -> JClass<'local> {
  a
}

fn main() {}
