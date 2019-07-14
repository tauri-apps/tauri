package webview

import (
	"image"
	"testing"
)

type foo struct {
	Result interface{}
}

func (f *foo) Foo1(a int, b float32) {
	f.Result = float64(a) + float64(b)
}
func (f *foo) Foo2(a []int, b [3]float32, c map[int]int) {
	f.Result = map[string]interface{}{"a": a, "b": b, "c": c}
}
func (f *foo) Foo3(a []image.Point, b struct{ Z int }) {
	f.Result = map[string]interface{}{"a": a, "b": b}
}

func TestBadBinding(t *testing.T) {
	x := 123
	for _, v := range []interface{}{
		nil,
		true,
		123,
		123.4,
		"hello",
		'a',
		make(chan struct{}, 0),
		func() {},
		map[string]string{},
		[]int{},
		[3]int{0, 0, 0},
		&x,
	} {
		if _, err := newBinding("test", v); err == nil {
			t.Errorf("should return an error: %#v", v)
		}
	}
}

func TestBindingCall(t *testing.T) {
	foo := &foo{}
	b, err := newBinding("test", foo)
	if err != nil {
		t.Fatal(err)
	}
	t.Run("Primitives", func(t *testing.T) {
		if !b.Call(`{"scope":"test","method":"Foo1","params":[3,4.5]}`) {
			t.Fatal()
		}
		if foo.Result.(float64) != 7.5 {
			t.Fatal(foo)
		}
	})

	t.Run("Collections", func(t *testing.T) {
		// Call with slices, arrays and maps
		if !b.Call(`{"scope":"test","method":"Foo2","params":[[1,2,3],[4.5,4.6,4.7],{"1":2,"3":4}]}`) {
			t.Fatal()
		}
		m := foo.Result.(map[string]interface{})
		if ints := m["a"].([]int); ints[0] != 1 || ints[1] != 2 || ints[2] != 3 {
			t.Fatal(foo)
		}
		if floats := m["b"].([3]float32); floats[0] != 4.5 || floats[1] != 4.6 || floats[2] != 4.7 {
			t.Fatal(foo)
		}
		if dict := m["c"].(map[int]int); len(dict) != 2 || dict[1] != 2 || dict[3] != 4 {
			t.Fatal(foo)
		}
	})

	t.Run("Structs", func(t *testing.T) {
		if !b.Call(`{"scope":"test","method":"Foo3","params":[[{"X":1,"Y":2},{"X":3,"Y":4}],{"Z":42}]}`) {
			t.Fatal()
		}
		m := foo.Result.(map[string]interface{})
		if p := m["a"].([]image.Point); p[0].X != 1 || p[0].Y != 2 || p[1].X != 3 || p[1].Y != 4 {
			t.Fatal(foo)
		}
		if z := m["b"].(struct{ Z int }); z.Z != 42 {
			t.Fatal(foo)
		}
	})

	t.Run("Errors", func(t *testing.T) {
		if b.Call(`{"scope":"foo"}`) || b.Call(`{"scope":"test", "method":"Bar"}`) {
			t.Fatal()
		}
		if b.Call(`{"scope":"test","method":"Foo1","params":["3",4.5]}`) {
			t.Fatal()
		}
	})
}
