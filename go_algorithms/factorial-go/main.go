// Sample run-helloworld is a minimal Cloud Run service.
package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
)

func main() {
	log.Print("starting server...")
	http.HandleFunc("/", handler)

	// Determine port for HTTP service.
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
		log.Printf("defaulting to port %s", port)
	}

	// Start HTTP server.
	log.Printf("listening on port %s", port)
	if err := http.ListenAndServe(":"+port, nil); err != nil {
		log.Fatal(err)
	}
}

func handler(w http.ResponseWriter, r *http.Request) {
// Get the "number" query parameter
	query := r.URL.Query()
	numberStr := query.Get("number")
	if numberStr == "" {
		http.Error(w, "Missing 'number' query parameter", http.StatusBadRequest)
		return
	}

	// Convert the number string to an integer
	number, err := strconv.Atoi(numberStr)
	if err != nil {
		http.Error(w, "Invalid 'number' query parameter", http.StatusBadRequest)
		return
	}

	// Calculate the factorial and respond
	fmt.Fprintf(w, "Factorial of %d is: %d\n", number, factorial(number))
}

func factorial(n int64) *big.Int {
    if n < 0 {
        return nil
    }
    r := big.NewInt(1)
    var f big.Int
    for i := int64(2); i <= n; i++ {
        r.Mul(r, f.SetInt64(i))
    }
    return r
}