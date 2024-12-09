package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
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
	maxNumberStr := query.Get("number")

    maxNumber := 100
    if maxNumberStr != "" {
		// Convert maxNumberStr to an integer
		var err error
		maxNumber, err = strconv.Atoi(maxNumberStr)
		if err != nil {
			http.Error(w, "Invalid number parameter", http.StatusBadRequest)
			return
		}
	}
    last := lastPrime(maxNumber)

	// Write the result to the response
	fmt.Fprintf(w, "The last prime number up to %d is: %d\n", maxNumber, last)
}

// Function to check if a number is prime
func isPrime(n int) bool {
	if n <= 1 {
		return false
	}
	for i := 2; i*i <= n; i++ {
		if n%i == 0 {
			return false
		}
	}
	return true
}

// Function to find the last prime number up to maxNumber
func lastPrime(maxNumber int) int {
	lastPrime := 0
	for i := 2; i <= maxNumber; i++ {
		if isPrime(i) {
			lastPrime = i
		}
	}
	return lastPrime
}