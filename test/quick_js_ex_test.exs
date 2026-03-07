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

      assert {:ok, 2} =
               QuickJSEx.eval(
                 rt,
                 "const m = new Map(); m.set('a', 1); m.set('b', 2); m.size"
               )

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

  describe "atob/btoa" do
    test "btoa encodes to base64" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "aGVsbG8="} = QuickJSEx.eval(rt, ~s[btoa("hello")])
      QuickJSEx.stop(rt)
    end

    test "atob decodes from base64" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "hello"} = QuickJSEx.eval(rt, ~s[atob("aGVsbG8=")])
      QuickJSEx.stop(rt)
    end

    test "roundtrip" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "test 123!"} = QuickJSEx.eval(rt, ~s[atob(btoa("test 123!"))])
      QuickJSEx.stop(rt)
    end
  end

  describe "TextEncoder/TextDecoder" do
    test "encode and decode roundtrip" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, "hello"} =
               QuickJSEx.eval(rt, ~s[new TextDecoder().decode(new TextEncoder().encode("hello"))])

      QuickJSEx.stop(rt)
    end

    test "unicode roundtrip" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, "Привет 🌍"} =
               QuickJSEx.eval(
                 rt,
                 ~s[new TextDecoder().decode(new TextEncoder().encode("Привет 🌍"))]
               )

      QuickJSEx.stop(rt)
    end

    test "encode returns correct byte lengths" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, 5} = QuickJSEx.eval(rt, ~s[new TextEncoder().encode("hello").length])
      assert {:ok, 12} = QuickJSEx.eval(rt, ~s[new TextEncoder().encode("Привет").length])
      QuickJSEx.stop(rt)
    end
  end

  describe "Buffer" do
    test "from string and toString" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "hello"} = QuickJSEx.eval(rt, ~s[Buffer.from("hello").toString()])
      QuickJSEx.stop(rt)
    end

    test "base64 roundtrip" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "aGVsbG8="} = QuickJSEx.eval(rt, ~s[Buffer.from("hello").toString("base64")])

      assert {:ok, "hello"} =
               QuickJSEx.eval(rt, ~s[Buffer.from("aGVsbG8=", "base64").toString()])

      QuickJSEx.stop(rt)
    end

    test "hex encoding" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "68656c6c6f"} = QuickJSEx.eval(rt, ~s[Buffer.from("hello").toString("hex")])

      assert {:ok, "hello"} =
               QuickJSEx.eval(rt, ~s[Buffer.from("68656c6c6f", "hex").toString()])

      QuickJSEx.stop(rt)
    end

    test "unicode" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "Привет"} = QuickJSEx.eval(rt, ~s[Buffer.from("Привет").toString()])
      QuickJSEx.stop(rt)
    end

    test "isBuffer" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, true} = QuickJSEx.eval(rt, ~s[Buffer.isBuffer(Buffer.from("x"))])
      assert {:ok, false} = QuickJSEx.eval(rt, ~s[Buffer.isBuffer("x")])
      QuickJSEx.stop(rt)
    end

    test "concat" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, "abcd"} =
               QuickJSEx.eval(
                 rt,
                 ~s{Buffer.concat([Buffer.from("ab"), Buffer.from("cd")]).toString()}
               )

      QuickJSEx.stop(rt)
    end

    test "read and write integers" do
      {:ok, rt} = QuickJSEx.start()

      assert {:ok, "deadbeef"} =
               QuickJSEx.eval(
                 rt,
                 "var b = Buffer.alloc(4); b.writeUInt32BE(0xDEADBEEF, 0); b.readUInt32BE(0).toString(16)"
               )

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

    test "call an async function" do
      {:ok, rt} = QuickJSEx.start()

      {:ok, _} =
        QuickJSEx.eval(rt, """
        async function fetchData(name) {
          return { greeting: "hello " + name };
        }
        """)

      assert {:ok, %{"greeting" => "hello world"}} = QuickJSEx.call(rt, "fetchData", ["world"])
      QuickJSEx.stop(rt)
    end

    test "call an async function returning a string" do
      {:ok, rt} = QuickJSEx.start()

      {:ok, _} =
        QuickJSEx.eval(rt, """
        async function render(name, props) {
          return "<div>" + name + ": " + JSON.stringify(props) + "</div>";
        }
        """)

      assert {:ok, html} = QuickJSEx.call(rt, "render", ["MyComponent", %{count: 0}])
      assert html =~ "<div>MyComponent"
      QuickJSEx.stop(rt)
    end

    test "call a Promise-returning function that rejects" do
      {:ok, rt} = QuickJSEx.start()

      {:ok, _} =
        QuickJSEx.eval(rt, """
        function failAsync() {
          return Promise.reject(new Error("async failure"));
        }
        """)

      assert {:error, "async failure"} = QuickJSEx.call(rt, "failAsync", [])
      QuickJSEx.stop(rt)
    end
  end

  describe "load_module" do
    test "loads an ES module and promotes exports to globalThis" do
      {:ok, rt} = QuickJSEx.start()

      module_code = """
      export function greet(name) {
        return "hello " + name;
      }

      export const VERSION = "1.0.0";
      """

      assert :ok = QuickJSEx.load_module(rt, "mylib", module_code)
      assert {:ok, "hello world"} = QuickJSEx.call(rt, "greet", ["world"])
      assert {:ok, "1.0.0"} = QuickJSEx.eval(rt, "VERSION")
      QuickJSEx.stop(rt)
    end

    test "loaded module exports are callable as async" do
      {:ok, rt} = QuickJSEx.start()

      module_code = """
      export async function render(name, props) {
        return "<div>" + name + ": " + JSON.stringify(props) + "</div>";
      }
      """

      assert :ok = QuickJSEx.load_module(rt, "ssr", module_code)
      assert {:ok, html} = QuickJSEx.call(rt, "render", ["App", %{page: 1}])
      assert html =~ "<div>App"
      QuickJSEx.stop(rt)
    end

    test "reports module syntax errors" do
      {:ok, rt} = QuickJSEx.start()
      assert {:error, msg} = QuickJSEx.load_module(rt, "bad", "export function {")
      assert is_binary(msg)
      QuickJSEx.stop(rt)
    end
  end

  describe "eval with export" do
    test "promotes exports to globalThis" do
      {:ok, rt} = QuickJSEx.start()

      {:ok, _} =
        QuickJSEx.eval(rt, """
        const greeting = "hi";
        export function greet(name) { return greeting + " " + name; }
        export const PI = 3.14;
        """)

      assert {:ok, "hi world"} = QuickJSEx.call(rt, "greet", ["world"])
      assert {:ok, 3.14} = QuickJSEx.eval(rt, "PI")
      QuickJSEx.stop(rt)
    end

    test "handles export { name } syntax" do
      {:ok, rt} = QuickJSEx.start()

      {:ok, _} =
        QuickJSEx.eval(rt, """
        function render(n) { return "<div>" + n + "</div>"; }
        export { render };
        """)

      assert {:ok, "<div>App</div>"} = QuickJSEx.call(rt, "render", ["App"])
      QuickJSEx.stop(rt)
    end
  end

  describe "reset" do
    test "clears all global state" do
      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "globalThis.x = 42")
      assert {:ok, 42} = QuickJSEx.eval(rt, "x")

      assert :ok = QuickJSEx.reset(rt)
      assert {:error, _} = QuickJSEx.eval(rt, "x")
      QuickJSEx.stop(rt)
    end

    test "clears loaded modules" do
      {:ok, rt} = QuickJSEx.start()
      :ok = QuickJSEx.load_module(rt, "m", "export function f() { return 1; }")
      assert {:ok, 1} = QuickJSEx.call(rt, "f", [])

      :ok = QuickJSEx.reset(rt)
      assert {:error, _} = QuickJSEx.call(rt, "f", [])
      QuickJSEx.stop(rt)
    end

    test "runtime is usable after reset" do
      {:ok, rt} = QuickJSEx.start()
      {:ok, _} = QuickJSEx.eval(rt, "globalThis.x = 1")
      :ok = QuickJSEx.reset(rt)

      {:ok, _} = QuickJSEx.eval(rt, "globalThis.y = 2")
      assert {:ok, 2} = QuickJSEx.eval(rt, "y")
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

  describe "browser_stubs option" do
    test "browser globals are undefined by default" do
      {:ok, rt} = QuickJSEx.start()
      assert {:ok, "undefined"} = QuickJSEx.eval(rt, "typeof window")
      assert {:ok, "undefined"} = QuickJSEx.eval(rt, "typeof document")
      QuickJSEx.stop(rt)
    end

    test "browser globals are available when enabled" do
      {:ok, rt} = QuickJSEx.start(browser_stubs: true)
      assert {:ok, "object"} = QuickJSEx.eval(rt, "typeof window")
      assert {:ok, "object"} = QuickJSEx.eval(rt, "typeof document")
      assert {:ok, nil} = QuickJSEx.eval(rt, "document.querySelector('div')")
      assert {:ok, "production"} = QuickJSEx.eval(rt, "process.env.NODE_ENV")
      assert {:ok, nil} = QuickJSEx.eval(rt, "localStorage.getItem('x')")
      assert {:ok, "function"} = QuickJSEx.eval(rt, "typeof MutationObserver")
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
