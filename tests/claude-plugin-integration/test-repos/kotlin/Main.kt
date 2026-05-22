package sample

class User(val name: String, val email: String)

class PaymentService {
    fun processPayment(user: User, amount: Double): Boolean {
        println("Processing $amount for ${user.name}")
        return true
    }
}

fun main() {
    val user = User("Alice Cooper", "alice@example.com")
    val service = PaymentService()
    service.processPayment(user, 150.0)
}
