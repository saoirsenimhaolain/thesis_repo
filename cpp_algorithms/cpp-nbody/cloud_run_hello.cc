// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// [START cloudrun_helloworld_service]
// [START cloud_run_hello_world]


// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by Elam Kolenovic
//
// Changes (2013-05-07)
//   - changed omp schedule for more even distribution of work
//   - changed loop variables to signed integer because msvc was complaining
//     when omp was enabled
//   - replaced std::copy and std::fill by one loop. slightly faster.
//   - swapped order of tests in for-i-loop. slightly faster.
//
// Changes (2013-04-19)
//   - using omp
//   - use buffer and fwrite at end instead of putchar
//   - pre-calculate cr0[]
//   - rename variables and use underscore before the index part of the name
//   - inverted bit tests, better performance under MSVC
//   - optional argument for file output, usefull in windows shell
//
// Changes (2013-04-07):
//   - removed unnecessary arrays, faster especially on 32 bits
//   - using putchar instead of iostreams, slightly faster
//   - using namespace std for readability
//   - replaced size_t with unsigned
//   - removed some includes

#include <google/cloud/functions/framework.h>
#include "google/cloud/functions/function.h"
#include "google/cloud/functions/http_request.h"
#include "google/cloud/functions/http_response.h"
#include <map>
#include <iostream>
#include <cmath>
#include <string>
#include <sstream>

typedef unsigned char Byte;

using namespace std;

namespace gcf = ::google::cloud::functions;


// Function to check if a number is prime
bool isPrime(int n) {
    if (n <= 1) {
        return false;
    }
    for (int i = 2; i <= std::sqrt(n); ++i) {
        if (n % i == 0) {
            return false;
        }
    }
    return true;
}

// Function to find the last prime number up to maxNumber
int lastPrime(int maxNumber) {
    int lastPrime = -1; // Default value if no prime is found
    for (int i = 2; i <= maxNumber; ++i) {
        if (isPrime(i)) {
            lastPrime = i;
        }
    }
    return lastPrime;
}

// Utility function to parse query parameters
std::map<std::string, std::string> parse_query(const std::string& target) {
    std::map<std::string, std::string> query_params;
    auto query_pos = target.find('?');
    if (query_pos == std::string::npos) {
        return query_params; // No query string
    }

    std::string query = target.substr(query_pos + 1);
    std::istringstream query_stream(query);
    std::string param;

    while (std::getline(query_stream, param, '&')) {
        auto equal_pos = param.find('=');
        if (equal_pos != std::string::npos) {
            std::string key = param.substr(0, equal_pos);
            std::string value = param.substr(equal_pos + 1);
            query_params[key] = value;
        }
    }
    return query_params;
}



auto hello_world_http() {
     return gcf::MakeFunction([](gcf::HttpRequest const& request) {
        // Default max number if no query parameter is provided
        int maxNumber = 100;

        // Parse the query string from the request target
        auto query_params = parse_query(request.target());
        auto it = query_params.find("max");
        if (it != query_params.end()) {
            try {
                maxNumber = std::stoi(it->second);
            } catch (...) {
                maxNumber = 100; // Fallback to default in case of parsing error
            }
        }

        // Calculate the last prime number
        int last_prime = lastPrime(maxNumber);

        // Prepare the response
        std::ostringstream response;
        response << "The last prime number up to " << maxNumber << " is: " << last_prime;

        return gcf::HttpResponse{}
            .set_header("Content-Type", "text/plain")
            .set_payload(response.str());
    });
}

int main(int argc, char* argv[]) {
  return gcf::Run(argc, argv, hello_world_http());
}

// [END cloud_run_hello_world]
// [END cloudrun_helloworld_service]