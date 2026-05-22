class User {
    let name: String
    let email: String

    init(name: String, email: String) {
        self.name = name
        self.email = email
    }
}

class PaymentService {
    func processPayment(user: User, amount: Double) -> Bool {
        print("Processing \(amount) for \(user.name)")
        return true
    }
}

func main() {
    let user = User(name: "Alice Cooper", email: "alice@example.com")
    let service = PaymentService()
    _ = service.processPayment(user: user, amount: 150)
}
