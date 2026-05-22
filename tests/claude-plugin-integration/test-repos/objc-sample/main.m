#import <Foundation/Foundation.h>

@interface User : NSObject
@property(nonatomic, copy) NSString *name;
@property(nonatomic, copy) NSString *email;
@end

@implementation User
@end

@interface PaymentService : NSObject
- (BOOL)processPayment:(User *)user amount:(double)amount;
@end

@implementation PaymentService
- (BOOL)processPayment:(User *)user amount:(double)amount {
    NSLog(@"Processing %f for %@", amount, user.name);
    return YES;
}
@end

int main(void) {
    User *user = [User new];
    user.name = @"Alice Cooper";
    user.email = @"alice@example.com";
    PaymentService *service = [PaymentService new];
    [service processPayment:user amount:150.0];
    return 0;
}
