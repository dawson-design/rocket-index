// Minimal Haxe fixture for testing

package minimal;

class Main {
    public static function helper():Int {
        return 42;
    }

    public static function mainFunction():Void {
        var x = helper();
        trace(x);
    }

    public static function callerA():Void {
        mainFunction();
    }

    public static function callerB():Void {
        mainFunction();
        helper();
    }
}

class MyClass {
    public var field:Int;

    public function new() {
        field = Main.helper();
    }

    public function method():Int {
        return field;
    }
}

class ChildClass extends MyClass {
    override public function method():Int {
        Main.mainFunction();
        return super.method();
    }
}
