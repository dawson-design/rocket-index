package sample;

class User {
    public var name:String;
    public var email:String;

    public function new(name:String, email:String) {
        this.name = name;
        this.email = email;
    }
}

class PaymentService {
    public function new() {}

    public function processPayment(user:User, amount:Float):Bool {
        trace('Processing $amount for ${user.name}');
        return true;
    }
}

class Main {
    public static function main():Void {
        var user = new User("Alice Cooper", "alice@example.com");
        var service = new PaymentService();
        service.processPayment(user, 150);
    }
}
