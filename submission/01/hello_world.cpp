#include <iostream>
#include <string>
#include <stdlib.h>

int main() {
    std::string name = getenv("HOSTNAME");
    std::cout << "Hello World from " << name << std::endl;
    return 0;
}
