// Minimal Kotlin fixture for testing

package minimal

fun helper(): Int {
    return 42
}

fun mainFunction() {
    val x = helper()
    println(x)
}

fun callerA() {
    mainFunction()
}

fun callerB() {
    mainFunction()
    helper()
}

open class MyClass {
    val field: Int = helper()

    open fun method(): Int {
        return field
    }
}

class ChildClass : MyClass() {
    override fun method(): Int {
        mainFunction()
        return super.method()
    }
}
