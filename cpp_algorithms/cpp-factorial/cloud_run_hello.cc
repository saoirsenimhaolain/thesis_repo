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
#include <google/cloud/functions/framework.h>
#include <cstdlib>
#include <string>
#include <iostream>
#include <vector>
#include <complex>
#include <algorithm>
#include <thread>
#include <climits>
#include <version>
#include <array>

namespace gcf = ::google::cloud::functions;


long long int factorial(long long int n)
{
   long long int r = 1;
   while(1<n)
       r *= n--;
   return r;
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

        int num = 12;

         auto query_params = parse_query(request.target());
        auto it = query_params.find("value");
        if (it != query_params.end()) {
            try {
                num = std::stoi(it->second);
            } catch (...) {
                num = 12; // Fallback to default in case of parsing error
            }
        }


 // Calculate factorial
    long long int fact_result = factorial(num);

    // Convert the result to a string
    std::string greeting = "Factorial(12): " + std::to_string(fact_result);

    return gcf::HttpResponse{}
        .set_header("Content-Type", "text/plain")
        .set_payload(greeting);
  });
}

int main(int argc, char* argv[]) {
  return gcf::Run(argc, argv, hello_world_http());
}

// [END cloud_run_hello_world]
// [END cloudrun_helloworld_service]