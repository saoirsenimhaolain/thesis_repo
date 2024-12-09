
#include <google/cloud/functions/framework.h>
#include <cstdlib>


#include <iostream>
#include <vector>
#include <algorithm>
#include <string>
#include <map>
#include <sstream>

using namespace std;

namespace gcf = ::google::cloud::functions;


static int64_t fact[32];

void initializeFact(int n)
{
    fact[0] = 1;
    for (auto i = 1; i <= n; ++i)
        fact[i] = i * fact[i - 1];
}

class Permutation
{
  public:
    Permutation(int n, int64_t start);
    void advance();
    int64_t countFlips() const;

  private:
     vector <int> count;
     vector <int8_t> current;

};

//
// Initialize the current value of a permutation
// and the cycle count values used to advance .
//
Permutation::Permutation(int n, int64_t start)
{
    count.resize(n);
    current.resize(n);

    // Initialize count
    for (auto i = n - 1; i >= 0; --i)
    {
        auto d = start / fact[i];
        start = start % fact[i];
        count[i] = d;
    }

    // Initialize current.
    for (auto i = 0; i < n; ++i)
        current[i] = i;

    for (auto i = n - 1; i >= 0; --i)
    {
        auto d = count[i];
        auto b = current.begin();
        rotate(b, b + d, b + i + 1);
    }
}

//
// Advance the current permutation to the next in sequence.
//
void Permutation::advance()
{
    for (auto i = 1; ;++i)
    {
        // Tried using std::rotate here but that was slower.
        auto first = current[0];
        for (auto j = 0; j < i; ++j)
            current[j] = current[j + 1];
        current[i] = first;

        ++(count[i]);
        if (count[i] <= i)
            break;
        count[i] = 0;
    }
}

//
// Count the flips required to flip 0 to the front of the vector.
//
// Other than minor cosmetic changes, the following routine is
// basically lifted from "fannkuch-redux C gcc #5"
//
inline int64_t Permutation::countFlips() const
{
    const auto n = current.size();
    auto flips = 0;
    auto first = current[0];
    if (first > 0)
    {
        flips = 1;

        int8_t temp[n];
        // Make a copy of current to work on.
        for (size_t i = 0; i < n; ++i)
            temp[i] = current[i];


        // Flip temp until the element at the first index is 0
        for (; temp[first] > 0; ++flips)
        {
            // Record the newFirst and restore the old
            // first at its new flipped position.
            const int8_t newFirst = temp[first];
            temp[first] = first;

            if (first > 2)
            {
                int64_t low = 1, high = first - 1;
                do
                {
                    swap(temp[low], temp[high]);
                    if (!(low + 3 <= high && low < 16))
                        break;
                    ++low;
                    --high;
                } while (1);
            }
            // Update first to newFirst that we recorded earlier.
            first = newFirst;
        }
    }
    return flips;
}

// Function to compute factorials and perform the main computation
std::pair<int64_t, int64_t> compute_factorials(int n, int blockCount) {
    initializeFact(n);

    // Adjust blockCount based on factorial of n
    if (blockCount > fact[n])
        blockCount = 1;
    const int64_t blockLength = fact[n] / blockCount;

    int64_t maxFlips = 0, checksum = 0;

    // Iterate over each block using OpenMP for parallelism
    #pragma omp parallel for reduction(max:maxFlips) reduction(+:checksum)
    for (int64_t blockStart = 0; blockStart < fact[n]; blockStart += blockLength) {
        Permutation permutation(n, blockStart);

        // Iterate over each permutation in the block
        auto index = blockStart;
        while (true) {
            const auto flips = permutation.countFlips();
            if (flips) {
                checksum += (index % 2 == 0) ? flips : -flips;
                if (flips > maxFlips)
                    maxFlips = flips;
            }

            if (++index == blockStart + blockLength)
                break;

            // Move to next permutation
            permutation.advance();
        }
    }

    return {checksum, maxFlips};
}



auto hello_world_http() {

  return gcf::MakeFunction([](gcf::HttpRequest const& request) {

    auto blockCount = 24;

        // Extract the query string from the URL
    std::string target = request.target();
    std::string query_params = target.substr(target.find('?') + 1);

    // Parse query parameters
    std::map<std::string, std::string> params;
    std::istringstream query_stream(query_params);
    std::string param;
    while (std::getline(query_stream, param, '&')) {
      size_t eq_pos = param.find('=');
      if (eq_pos != std::string::npos) {
        params[param.substr(0, eq_pos)] = param.substr(eq_pos + 1);
      }
    }

    // Get the 'n' parameter or default to 12 if not provided
    int n = 12;  // Default value
    if (params.find("n") != params.end()) {
      try {
        n = std::stoi(params["n"]);
      } catch (...) {
        // Handle invalid input (if 'n' is not a valid integer)
        n = 12;  // Default value if invalid
      }
    }

   auto [checksum, maxFlips] = compute_factorials(n, blockCount);


    // Append checksum and maxFlips to the response
    std::string response = "Checksum: " + std::to_string(checksum) + "\n";
    response += "Pfannkuchen(" + std::to_string(n) + ") = " + std::to_string(maxFlips) + "\n";

    return gcf::HttpResponse{}
        .set_header("Content-Type", "text/plain")
        .set_payload(response);
  });
}

int main(int argc, char* argv[]) {
  return gcf::Run(argc, argv, hello_world_http());
}

