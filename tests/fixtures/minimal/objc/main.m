// Minimal Objective-C fixture for testing

#import <Foundation/Foundation.h>

int helper(void) {
    return 42;
}

void mainFunction(void) {
    int x = helper();
    NSLog(@"%d", x);
}

void callerA(void) {
    mainFunction();
}

void callerB(void) {
    mainFunction();
    helper();
}

@interface MyClass : NSObject
- (instancetype)init;
- (int)method;
@end

@implementation MyClass
- (instancetype)init {
    self = [super init];
    if (self) {
        helper();
    }
    return self;
}

- (int)method {
    return helper();
}
@end

@interface ChildClass : MyClass
@end

@implementation ChildClass
- (int)method {
    mainFunction();
    return [super method];
}
@end
