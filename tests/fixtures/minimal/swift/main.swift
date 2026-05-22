// Minimal Swift fixture for testing

func helper() -> Int {
    return 42
}

func mainFunction() {
    let x = helper()
    print(x)
}

func callerA() {
    mainFunction()
}

func callerB() {
    mainFunction()
    _ = helper()
}

class MyClass {
    let field: Int

    init() {
        self.field = helper()
    }

    func method() -> Int {
        return field
    }
}

class ChildClass: MyClass {
    override func method() -> Int {
        mainFunction()
        return super.method()
    }
}
