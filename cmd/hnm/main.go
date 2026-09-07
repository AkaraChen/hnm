package main

import (
	"fmt"
	"os"

	"github.com/AkaraChen/hnm/internal/generated"
	"github.com/AkaraChen/hnm/internal/hooks"
	"github.com/spf13/cobra"
)

func newCommand() *cobra.Command {
	root := generated.New()
	for _, cmd := range root.Commands() {
		if cmd.Name() != "init" {
			continue
		}
		run := cmd.RunE
		cmd.Args = cobra.NoArgs
		cmd.RunE = func(cmd *cobra.Command, args []string) error {
			scope, err := cmd.Flags().GetString("scope")
			if err != nil {
				return err
			}
			if scope != "project" {
				return run(cmd, args)
			}
			force, err := cmd.Flags().GetBool("force")
			if err != nil {
				return err
			}
			files, err := hooks.Prepare(force)
			if err != nil {
				return err
			}
			if err := run(cmd, args); err != nil {
				return err
			}
			for _, file := range files {
				if err := file.Write(); err != nil {
					return fmt.Errorf("install %s: %w", file.Path, err)
				}
				fmt.Fprintf(cmd.ErrOrStderr(), "installed %s\n", file.Path)
			}
			fmt.Fprintln(cmd.ErrOrStderr(), "Documentation review hooks configured (Python 3 and Git required). Review/trust hooks in Codex /hooks; restart Claude Code to load settings changes.")
			return nil
		}
	}
	return root
}

func main() {
	if err := newCommand().Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "hnm:", err)
		os.Exit(1)
	}
}
