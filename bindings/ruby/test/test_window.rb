# frozen_string_literal: true

# Native windows: only where a display is (BUI_TEST_WINDOWS=1 asks for it).
#
# SPDX-License-Identifier: GPL-2.0-or-later

require "test_helper"

class TestWindow < Minitest::Test
  def setup
    skip "set BUI_TEST_WINDOWS=1 with a display to test native windows" unless ENV["BUI_TEST_WINDOWS"]
  end

  def test_run_until_closed
    closes = 0
    other_thread = nil
    win = BUI::Window.new("Test", width: 200, height: 100) do
      label "Closing soon"
    end
    win.on_close { closes += 1 }
    Thread.new { sleep 0.05; win.update { other_thread = :ran } }
    win.timer(200) { win.close }
    BUI.run
    assert_equal :ran, other_thread
    refute win.open?
    win.free
    assert win.freed?
  end

  def test_an_error_stops_the_loop
    win = BUI::Window.new("Test", width: 200, height: 100)
    win.timer(10) { raise ArgumentError, "from a timer" }
    err = assert_raises(ArgumentError) { BUI.run }
    assert_equal "from a timer", err.message
  ensure
    win&.free
  end

  def test_an_interrupt_ends_the_loop
    win = BUI::Window.new("Test", width: 200, height: 100)
    # what Ctrl+C does: an exception raised into the thread that waits in the loop
    Thread.new { sleep 0.1; Thread.main.raise(Interrupt) }
    assert_raises(Interrupt) { BUI.run }
  ensure
    win&.free
  end

  def test_free_inside_a_callback
    win = BUI::Window.new("Test", width: 200, height: 100)
    win.timer(10) { win.free }
    BUI.run
    assert win.freed?
  end

  def test_a_window_freed_by_a_failing_timer_is_freed
    before = BUI.references
    win = BUI::Window.new("Test", width: 200, height: 100)
    win.timer(10) do
      win.free
      raise ArgumentError, "after the free"
    end
    assert_raises(ArgumentError) { BUI.run }
    assert win.freed?
    assert win.context.freed?
    assert_equal before, BUI.references
  end

  def test_a_failing_builder_leaves_nothing
    before = BUI.references
    assert_raises(ArgumentError) do
      BUI::Window.new("Test", width: 200, height: 100) { button "x", no_such_key: 1 }
    end
    assert_equal before, BUI.references
  end

  def test_a_root_you_pass_stays_yours
    root = BUI::Node.new("column")
    win = BUI::Window.new("Test", width: 200, height: 100, root: root, show: false)
    win.free
    refute root.freed?
  ensure
    root&.free
  end

  def test_a_shared_context_outlives_the_borrower
    owner = BUI::Window.new("Owner", width: 200, height: 100, show: false)
    borrower = BUI::Window.new("Borrower", width: 200, height: 100, context: owner.context, show: false)
    assert_raises(BUI::Error) { owner.free } # nothing freed: it can be tried again
    refute owner.freed?
    borrower.free
    owner.free
    assert owner.freed? && owner.context.freed?
  end
end
