# frozen_string_literal: true

# What the binding refuses at its boundary, and when errors come out.
#
# SPDX-License-Identifier: GPL-2.0-or-later

require "test_helper"

class TestBoundaries < Minitest::Test
  include Headless

  def test_the_kit_belongs_to_one_thread
    node = BUI::Node.new("label")
    err = Thread.new do
      node.text
    rescue BUI::Error => e
      e
    end.value
    assert_kind_of BUI::Error, err
    assert_raises(BUI::Error) { Thread.new { BUI::Node.new("label") }.join }
    # images are the kit's thread-safe part: decoding elsewhere is fine
    img = Thread.new { BUI::Image.from_rgba("\x00".b * 16, 2, 2) }.value
    assert_equal 2, img.width
  ensure
    node&.free
  end

  def test_conversions_run_no_ruby_code
    node = BUI::Node.new("label")
    sneaky = Object.new
    def sneaky.to_str = "text"
    def sneaky.to_int = 1
    assert_raises(TypeError) { node.set_str("text", sneaky) }
    assert_raises(TypeError) { node.set_num("value", sneaky) }
  ensure
    node&.free
  end

  def test_a_notifying_setter_raises_its_handlers_error
    root = BUI::Node.new("column")
    slider = BUI::Node.new("slider").set(min: 0, max: 10)
    root << slider
    slider.on_change { raise IOError, "from the handler" }
    show(root) do
      err = assert_raises(IOError) { slider.set_num("set_value", 5) }
      assert_equal "from the handler", err.message
      slider.on_change {}
      assert slider.set_num("set_value", 6) # nothing left over from before
    end
  end

  def test_a_render_holds_its_raster
    raster = BUI::Raster.new(64, 64)
    root = BUI::Node.new("column")
    root.on(:configure) { raster.free } # runs inside the render, which draws into it
    ctx = BUI::Context.new(root)
    assert_raises(BUI::Error) { ctx.render(raster, full: true) }
    assert_equal 64, raster.width # still there
  ensure
    ctx&.free
    root&.free
  end

  def test_a_raster_a_context_keeps_is_not_freed
    root = BUI::Node.new("column")
    ctx = BUI::Context.new(root)
    first = BUI::Raster.new(32, 32)
    ctx.render(first, full: true)
    assert_raises(BUI::Error) { first.free } # the eyedropper may read it
    ctx.render(BUI::Raster.new(32, 32), full: true)
    first.free # another raster is the last frame now
    assert_raises(BUI::FreedError) { first.width }
  ensure
    ctx&.free
    root&.free
  end

  # Building menu items asks each action whether it responds to call, which is Ruby:
  # what it frees is looked at after it, not used freed.
  def test_a_menu_action_that_frees_the_menubutton
    mb = BUI::Node.new("menubutton")
    action = Object.new
    action.define_singleton_method(:respond_to?) { |*| mb.free || true }
    assert_raises(BUI::FreedError) { mb.menu = [{ text: "x", action: action }] }
    assert mb.freed?
  end

  def test_a_menu_action_that_frees_the_context_or_the_owner
    root = BUI::Node.new("column")
    owner = BUI::Node.new("button")
    root << owner
    ctx = BUI::Context.new(root)
    frees_owner = Object.new
    frees_owner.define_singleton_method(:respond_to?) { |*| owner.free || true }
    assert_raises(BUI::FreedError) { ctx.open_menu([{ text: "x", action: frees_owner }], x: 1, y: 1, owner: owner) }
    frees_ctx = Object.new
    frees_ctx.define_singleton_method(:respond_to?) { |*| ctx.free || true }
    assert_raises(BUI::FreedError) { ctx.open_menu([{ text: "x", action: frees_ctx }], x: 1, y: 1) }
    assert ctx.freed?
  ensure
    ctx&.free
    root&.free
  end
end
