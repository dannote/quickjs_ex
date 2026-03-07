defmodule QuickJSExTest do
  use ExUnit.Case
  doctest QuickJSEx

  test "greets the world" do
    assert QuickJSEx.hello() == :world
  end
end
