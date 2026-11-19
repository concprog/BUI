// A counter, MyGo style: the view builds the window from the state, and runs
// again after input. The arrow keys count too.
//
//	go run ./examples/counter
//
// SPDX-License-Identifier: GPL-2.0-or-later
package main

import (
	"log"

	"github.com/ppmpreetham/BUI/bindings/go/bui"
	"github.com/ppmpreetham/BUI/bindings/go/ui"
)

// counter is the state the window shows.
type counter struct{ n int }

// view builds the interface from the state, on the main thread, whenever the
// window needs a frame.
func (s *counter) view(c *ui.Context) {
	if c.Shortcut(0, ui.KeyUp) {
		s.n++
	}
	if c.Shortcut(0, ui.KeyDown) {
		s.n--
	}
	t := c.Theme()
	ui.Column(c).Fill().Center().Gap(16).Children(func() {
		ui.Textf(c, "%d", s.n).FontSize(56).Bold()
		ui.Row(c).Gap(8).Children(func() {
			if ui.Button(c.Key("decrement"), "−").Label("Decrement").Width(44).Clicked() {
				s.n--
			}
			if ui.Button(c.Key("reset"), "Reset").Disabled(s.n == 0).Clicked() {
				s.n = 0
			}
			if ui.PrimaryButton(c.Key("increment"), "+").Label("Increment").Width(44).Clicked() {
				s.n++
			}
		})
		ui.Text(c, "↑ and ↓ count too").FontSize(12).TextColor(t.TextMuted)
	})
}

func main() {
	s := &counter{}
	bui.App.WhenReady(func() {
		bui.NewWindow(bui.WindowOptions{
			Title:   "Counter",
			Width:   360,
			Height:  260,
			Content: ui.View(s.view),
		})
	})
	if err := bui.App.Run(); err != nil {
		log.Fatal(err)
	}
}
