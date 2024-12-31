package tokenizer_test

import (
	"fmt"
	"strings"
	"testing"

	"golang.org/x/net/html"
)

func TestTokenizer(t *testing.T) {
	input := "<script>foo</badtag></script>"
	tokenizer := html.NewTokenizer(strings.NewReader(input))
	for {
		tokenizer.Next()
		fmt.Printf("%+v\n", tokenizer.Token())
		if tokenizer.Err() != nil {
			fmt.Printf("error: %v\n", tokenizer.Err())
			break
		}
	}
}
