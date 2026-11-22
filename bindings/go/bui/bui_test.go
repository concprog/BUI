// SPDX-License-Identifier: GPL-2.0-or-later

package bui

import (
	"runtime"
	"strings"
	"testing"
	"time"
)

func TestVersion(t *testing.T) {
	if v := Version(); strings.Count(v, ".") != 2 {
		t.Errorf("version %q", v)
	}
}

func TestPropertiesAndTree(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	s := NewNode("slider").Set("label", "Size").Set("min", 0).Set("max", 10).Set("value", 2.5)
	d := NewNode("dropdown").Set("items", []string{"A", "B"})
	root.Add(s).Add(d)
	if s.Type() != "slider" || s.Str("label") != "Size" || s.Num("value") != 2.5 {
		t.Errorf("slider: %q %q %v", s.Type(), s.Str("label"), s.Num("value"))
	}
	if got := d.Strs("items"); len(got) != 2 || got[1] != "B" {
		t.Errorf("items %q", got)
	}
	if root.ChildCount() != 2 || root.Child(1) != d || d.Parent() != root {
		t.Errorf("tree")
	}
	if s.SetNum("no_such_key", 1) {
		t.Errorf("an unknown key was taken")
	}
	root.Remove(d)
	d.Free()
	if root.ChildCount() != 1 {
		t.Errorf("remove")
	}
}

func TestEventsHeadless(t *testing.T) {
	root := NewNode("column").Set("padding", 8)
	defer root.Free()
	b := NewNode("button").Set("text", "Go")
	c := NewNode("checkbox").Set("text", "On")
	root.Add(b).Add(c)
	clicks, changes, seen := 0, 0, 0
	b.OnClick(func(e Event) {
		if e.Node == b {
			clicks++
		}
	})
	c.OnChange(func(e Event) { changes++ })
	ctx := NewContext(root)
	defer ctx.Free()
	ctx.OnEvent(func(Event) { seen++ })
	r := NewRaster(200, 120)
	defer r.Free()
	if ctx.Render(r, 200, 120, true) == 0 {
		t.Fatalf("nothing rendered")
	}
	x, y := b.Rect().Center()
	ctx.Click(x, y, 0)
	x, y = c.Rect().Center()
	ctx.Click(x, y, 0)
	if clicks != 1 || changes != 1 || c.Num("value") != 1 || seen < 2 {
		t.Errorf("clicks %d, changes %d, value %v, seen %d", clicks, changes, c.Num("value"), seen)
	}
	if ctx.HitAt(x, y) != c {
		t.Errorf("HitAt")
	}
	// some pixel differs from the first: the frame is drawn
	px := r.Pixels()
	diff := false
	for _, p := range px {
		if p != px[0] {
			diff = true
			break
		}
	}
	if !diff {
		t.Errorf("a blank frame")
	}
}

func TestMenuActionsAndBindings(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	mb := NewNode("menubutton").Set("text", "File")
	root.Add(mb)
	opened := 0
	mb.SetMenu([]MenuItem{{Text: "Open", Shortcut: "Ctrl O", Action: func() { opened++ }}, {}, {Text: "Quit"}})
	ctx := NewContext(root)
	defer ctx.Free()
	r := NewRaster(300, 200)
	defer r.Free()
	ctx.Render(r, 300, 200, true)
	x, y := mb.Rect().Center()
	ctx.Click(x, y, 0)
	ctx.Render(r, 300, 200, false)
	pops := ctx.Popups()
	if len(pops) != 1 {
		t.Fatalf("%d popups after clicking the menu button", len(pops))
	}
	rows := MenuRows(pops[0])
	if len(rows) < 2 || rows[0] != "Open" {
		t.Fatalf("menu rows %q", rows)
	}
	x, y = MenuRowRect(pops[0], 0).Center()
	ctx.Motion(x, y, 0)
	ctx.Click(x, y, 0)
	if opened != 1 {
		t.Errorf("the Open action ran %d times", opened)
	}

	saved := 0
	ctx.Bind("ctrl+s", func() { saved++ })
	ctx.Key("", "s", Ctrl)
	if saved != 1 {
		t.Errorf("ctrl+s ran %d times", saved)
	}
}

func TestCanvasAndTheme(t *testing.T) {
	cv := NewNode("canvas")
	defer cv.Free()
	cv.AddItem(CanvasItem{ID: 1, Type: ItemRect, Coords: []float32{0, 0, 10, 10}, Fill: Color{1, 0, 0, 1}})
	cv.SetItem(CanvasItem{ID: 2, Type: ItemText, Coords: []float32{5, 5}, Text: "hi", Fill: Color{1, 1, 1, 1}})
	if cv.ItemCount() != 2 {
		t.Errorf("%d items", cv.ItemCount())
	}
	cv.DeleteItem(-1)
	if cv.ItemCount() != 0 {
		t.Errorf("delete all")
	}

	if !SetTheme("light") || ThemeMode() != "light" {
		t.Errorf("light theme")
	}
	if c, ok := ThemeColor("text"); !ok || c.A == 0 {
		t.Errorf("theme text color %v %v", c, ok)
	}
	SetTheme("dark")
	if v, ok := EvalNumber("2*3+1"); !ok || v != 7 {
		t.Errorf("EvalNumber %v %v", v, ok)
	}
	if Icon("FILE_FOLDER") == 0 {
		t.Errorf("no FILE_FOLDER icon")
	}
}

func TestFreeReleasesTheTree(t *testing.T) {
	before := len(nodes)
	root := NewNode("column")
	child := NewNode("button").Set("text", "x")
	root.Add(child)
	root.Child(0) // a wrapper looked up again is the same one
	root.Free()
	if !root.Freed() || !child.Freed() {
		t.Errorf("Free left the root (%v) or its child (%v) alive", root.Freed(), child.Freed())
	}
	if len(nodes) != before {
		t.Errorf("%d wrappers left after Free", len(nodes)-before)
	}
	root.Free() // a second Free does nothing
}

func TestBoundaryChecks(t *testing.T) {
	if ImageFromRGBA(make([]byte, 16), 2, 2, 4) != nil {
		t.Errorf("a stride shorter than a row was taken")
	}
	img := ImageFromRGBA(make([]byte, 2*8+4*2), 2, 2, 16) // a tight last row
	if img == nil {
		t.Fatalf("a buffer ending with a tight last row was refused")
	}
	img.Free()
	n := NewNode("slider")
	defer n.Free()
	n.Set("max", uint8(7)).Set("value", uint16(3))
	if n.Num("value") != 3 {
		t.Errorf("Set with small unsigned types")
	}
}

func TestMoveKeepsTheChild(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	a, b, c := NewNode("label"), NewNode("label"), NewNode("label")
	root.Add(a).Add(b).Add(c)
	if !root.Move(c, 0) || root.Child(0) != c || root.Child(1) != a || root.Child(2) != b {
		t.Errorf("Move to the front")
	}
	if !root.Move(c, -1) || root.Child(2) != c {
		t.Errorf("Move to the end")
	}
	if NewNode("label").Move(a, 0) {
		t.Errorf("Move of another node's child")
	}
}

func TestFreeInsideCallbacks(t *testing.T) {
	if !WindowSupported() {
		t.Skip("no native windows here")
	}
	runtime.LockOSThread() // the windows and their loop stay on one thread
	defer runtime.UnlockOSThread()
	pumpUntilFreed := func(w *Window) {
		for deadline := time.Now().Add(5 * time.Second); w.h != nil && time.Now().Before(deadline); {
			Pump()
			time.Sleep(time.Millisecond)
		}
		if w.h != nil {
			t.Fatalf("the window was never freed")
		}
		if _, ok := windows[w.id]; ok || len(later) != 0 {
			t.Errorf("the free left the window table or the queue behind")
		}
	}

	// from its own timer, with a loop nested in it
	w := NewWindow(WindowOptions{Title: "free", Width: 120, Height: 80, Hidden: true})
	if w == nil {
		t.Skip("no window")
	}
	fired := false
	w.Timer(1, func() {
		fired = true
		w.Free()
		Pump() // a loop nested in the callback frees nothing either
		if w.h == nil {
			t.Errorf("freed inside its own timer, with the kit still using it")
		}
	})
	pumpUntilFreed(w)
	if !fired {
		t.Errorf("the timer never fired")
	}

	// from a node's handler, inside the kit's dispatch on the window's context
	b := NewNode("button").Set("text", "Close")
	root := NewNode("column").Add(b)
	defer root.Free()
	w = NewWindow(WindowOptions{Root: root, Width: 120, Height: 80, Hidden: true})
	inside := true
	b.OnClick(func(Event) {
		w.Free()
		inside = w.h == nil
	})
	r := NewRaster(120, 80)
	defer r.Free()
	w.Context().Render(r, 120, 80, true)
	x, y := b.Rect().Center()
	w.Context().Click(x, y, 0) // the free runs as the headless call returns
	if inside {
		t.Errorf("freed inside a handler of its context's dispatch")
	}
	pumpUntilFreed(w)
	if root.Freed() || b.Freed() {
		t.Errorf("the window freed the Root it was given")
	}
}

func TestFreeNodeInsideItsHandler(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	c := NewNode("checkbox").Set("label", "Gone")
	root.Add(c)
	inside := true
	c.OnChange(func(Event) {
		c.Free()
		inside = c.Freed()
	})
	ctx := NewContext(root)
	defer ctx.Free()
	r := NewRaster(200, 80)
	defer r.Free()
	ctx.Render(r, 200, 80, true)
	x, y := c.Rect().Center()
	ctx.Click(x, y, 0) // the kit reads the check box after its handler returns
	if inside {
		t.Errorf("freed inside its handler, with the kit still using it")
	}
	if !c.Freed() || root.ChildCount() != 0 || len(later) != 0 {
		t.Errorf("the free was never run once the handler returned")
	}
}

func TestFreeInsideAHandlerWhileRequestRenders(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	c := NewNode("checkbox").Set("label", "Gone")
	root.Add(c)
	c.OnChange(func(Event) { c.Free() })
	ctx := NewContext(root)
	defer ctx.Free()
	r := NewRaster(200, 80)
	defer r.Free()
	ctx.Render(r, 200, 80, true)
	clicking, early := false, false
	ctx.OnRequest(func() { // the kit asks for a render inside the press
		ctx.Render(r, 200, 80, false)
		early = early || (clicking && c.Freed())
	})
	x, y := c.Rect().Center()
	clicking = true
	ctx.Click(x, y, 0)
	clicking = false
	if early {
		t.Errorf("a render the kit asked for freed the node it was still using")
	}
	if !c.Freed() || root.ChildCount() != 0 || len(later) != 0 {
		t.Errorf("the free was never run once the click returned")
	}
}

func TestPanelMoveEvent(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	p := [2]*Node{NewNode("panel").Set("text", "A"), NewNode("panel").Set("text", "B")}
	root.Add(p[0]).Add(p[1])
	from, to := -1.0, -1.0
	p[0].On(EventMove, func(e Event) { from, to = e.A, e.B })
	ctx := NewContext(root)
	defer ctx.Free()
	r := NewRaster(300, 400)
	defer r.Free()
	ctx.Render(r, 300, 400, true)
	a, b := p[0].Rect(), p[1].Rect()
	gx, gy := a.X+a.W-8, a.Y+8 // the first panel's grip, dragged below the second
	ctx.Motion(gx, gy, 0)
	ctx.Press(gx, gy, 0)
	ctx.Motion(gx, gy+10, 0)
	ctx.Motion(gx, b.Y+b.H-2, 0)
	ctx.Release(gx, b.Y+b.H-2, 0)
	if root.Child(1) != p[0] || from != 0 || to != 1 {
		t.Errorf("the drag moved the panel from %v to %v", from, to)
	}
	defer func() {
		if recover() == nil {
			t.Errorf("On took an unknown event kind")
		}
	}()
	p[0].On(EventKind(eventKindCount), func(Event) {})
}

func TestFreeContextInsideAHandler(t *testing.T) {
	root := NewNode("column")
	defer root.Free()
	b := NewNode("button").Set("text", "Done")
	root.Add(b)
	ctx := NewContext(root)
	defer ctx.Free()
	inside := true
	b.OnClick(func(Event) {
		ctx.Free()
		inside = ctx.h == nil
	})
	r := NewRaster(120, 80)
	defer r.Free()
	ctx.Render(r, 120, 80, true)
	x, y := b.Rect().Center()
	ctx.Click(x, y, 0) // the kit goes on using the context after the handler
	if inside {
		t.Errorf("freed inside a handler of its own dispatch")
	}
	if _, ok := contexts[ctx.id]; ctx.h != nil || ok || len(later) != 0 {
		t.Errorf("the free was never run once the headless call returned")
	}

	if !WindowSupported() {
		return
	}
	w := NewWindow(WindowOptions{Width: 120, Height: 80, Hidden: true})
	if w == nil {
		return
	}
	defer w.Free()
	func() {
		defer func() {
			if recover() == nil {
				t.Errorf("a context a window shows was freed")
			}
		}()
		w.Context().Free()
	}()
	if w.Context().h == nil {
		t.Errorf("the refused free freed the context")
	}
}
