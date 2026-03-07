defmodule QuickJSExTest do
  use ExUnit.Case

  describe "basic eval" do
    test "arithmetic" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, 3} = QuickJSEx.eval(rt, "1 + 2")
      assert {:ok, 6} = QuickJSEx.eval(rt, "2 * 3")
      QuickJSEx.stop(rt)
    end

    test "strings" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "hello world"} = QuickJSEx.eval(rt, "'hello' + ' ' + 'world'")
      QuickJSEx.stop(rt)
    end

    test "booleans" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, true} = QuickJSEx.eval(rt, "true")
      assert {:ok, false} = QuickJSEx.eval(rt, "false")
      QuickJSEx.stop(rt)
    end

    test "null and undefined" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, nil} = QuickJSEx.eval(rt, "null")
      assert {:ok, nil} = QuickJSEx.eval(rt, "undefined")
      QuickJSEx.stop(rt)
    end

    test "objects" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, %{"a" => 1, "b" => [2, 3]}} = QuickJSEx.eval(rt, "({a: 1, b: [2, 3]})")
      QuickJSEx.stop(rt)
    end

    test "arrays" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, [1, 2, 3]} = QuickJSEx.eval(rt, "[1, 2, 3]")
      QuickJSEx.stop(rt)
    end
  end

  describe "ES2023+ features" do
    test "arrow functions" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, 10} = QuickJSEx.eval(rt, "const add = (a, b) => a + b; add(4, 6)")
      QuickJSEx.stop(rt)
    end

    test "template literals" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "hello world"} = QuickJSEx.eval(rt, "const x = 'world'; `hello ${x}`")
      QuickJSEx.stop(rt)
    end

    test "destructuring" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, 3} = QuickJSEx.eval(rt, "const {a, b} = {a: 1, b: 2}; a + b")
      QuickJSEx.stop(rt)
    end

    test "Map and Set" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, 2} = QuickJSEx.eval(rt, "const m = new Map(); m.set('a', 1); m.set('b', 2); m.size")
      assert {:ok, 3} = QuickJSEx.eval(rt, "const s = new Set([1, 2, 3, 2, 1]); s.size")
      QuickJSEx.stop(rt)
    end

    test "Proxy" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, 42} =
               QuickJSEx.eval(rt, """
               const handler = { get: (target, prop) => prop === 'answer' ? 42 : target[prop] };
               const p = new Proxy({}, handler);
               p.answer
               """)

      QuickJSEx.stop(rt)
    end

    test "async/await with Promise" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, "done"} =
               QuickJSEx.eval(rt, """
               async function work() { return "done"; }
               await work()
               """)

      QuickJSEx.stop(rt)
    end

    test "Promise.all" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, [1, 2, 3]} =
               QuickJSEx.eval(rt, """
               await Promise.all([
                 Promise.resolve(1),
                 Promise.resolve(2),
                 Promise.resolve(3)
               ])
               """)

      QuickJSEx.stop(rt)
    end
  end

  describe "state persistence" do
    test "global state persists across evals" do
      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "globalThis.counter = 0")
      {:ok, _} = QuickJSEx.eval(rt, "counter += 1")
      {:ok, _} = QuickJSEx.eval(rt, "counter += 1")
      assert {:ok, 2} = QuickJSEx.eval(rt, "counter")
      QuickJSEx.stop(rt)
    end

    test "functions persist across evals" do
      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "function double(x) { return x * 2; }")
      assert {:ok, 10} = QuickJSEx.eval(rt, "double(5)")
      QuickJSEx.stop(rt)
    end
  end

  describe "call function" do
    test "call a global function with args" do
      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "function add(args) { return args[0] + args[1]; }")
      assert {:ok, 7} = QuickJSEx.call(rt, "add", [[3, 4]])
      QuickJSEx.stop(rt)
    end
  end

  describe "error handling" do
    test "syntax error" do
      {:ok, rt} = QuickJSEx.start()
      assert {:error, msg} = QuickJSEx.eval(rt, "function(")
      assert is_binary(msg)
      QuickJSEx.stop(rt)
    end

    test "runtime error" do
      {:ok, rt} = QuickJSEx.start()
      assert {:error, msg} = QuickJSEx.eval(rt, "undefined_var.property")
      assert is_binary(msg)
      QuickJSEx.stop(rt)
    end

    test "throw" do
      {:ok, rt} = QuickJSEx.start()
      assert {:error, msg} = QuickJSEx.eval(rt, "throw new Error('boom')")
      assert msg =~ "boom"
      QuickJSEx.stop(rt)
    end
  end

  describe "isolation" do
    test "separate runtimes are isolated" do
      {:ok, rt1} = QuickJSEx.start()
      {:ok, rt2} = QuickJSEx.start()

      {:ok, _} = QuickJSEx.eval(rt1, "globalThis.x = 'from_rt1'")
      assert {:ok, "from_rt1"} = QuickJSEx.eval(rt1, "x")

      assert {:error, _} = QuickJSEx.eval(rt2, "x")

      QuickJSEx.stop(rt1)
      QuickJSEx.stop(rt2)
    end
  end
end
