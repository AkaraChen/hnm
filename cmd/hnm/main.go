package main

import (
	"fmt"
	"os"

	initcmd "github.com/AkaraChen/hnm/internal/init"
	"github.com/AkaraChen/hnm/internal/stack"
	"github.com/spf13/cobra"
)

func main() {
	if err := rootCmd().Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

func rootCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "hnm",
		Version: "0.1.0",
		Short:   "Install the agent documentation harness into a project",
		Long:    "hnm writes AGENTS.md, docs/{prd,adr,spec}, feature-dev and git-commit skills, and commit-time doc review hooks so AI agents follow a consistent documentation harness.",
	}
	cmd.AddCommand(initCommand())
	return cmd
}

func initCommand() *cobra.Command {
	var name string
	var stackName string
	var force bool
	var dryRun bool
	cmd := &cobra.Command{
		Use:   "init [PATH]",
		Short: "Write the full harness into a target directory",
		Args:  cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			target := "."
			if len(args) == 1 {
				target = args[0]
			}
			st, err := stack.Parse(stackName)
			if err != nil {
				return err
			}
			opts := initcmd.Options{
				Target:      target,
				ProjectName: initcmd.ResolveProjectName(name, target),
				Stack:       st,
				Force:       force,
				DryRun:      dryRun,
			}
			report, err := initcmd.Run(opts)
			if err != nil {
				return fmt.Errorf("init failed: %w", err)
			}
			if opts.DryRun {
				fmt.Printf("dry-run harness for `%s` (stack: %s)\n", opts.ProjectName, opts.Stack)
			} else {
				fmt.Printf("installed harness for `%s` (stack: %s) into %s\n", opts.ProjectName, opts.Stack, opts.Target)
			}
			for _, line := range report.SummaryLines() {
				fmt.Printf("  %s\n", line)
			}
			return nil
		},
	}
	cmd.Flags().StringVarP(&name, "name", "n", "", "Project name embedded in AGENTS.md and docs/spec.md")
	cmd.Flags().StringVarP(&stackName, "stack", "s", "generic", "Commands-section preset for AGENTS.md")
	cmd.Flags().BoolVarP(&force, "force", "f", false, "Overwrite existing files and replace incorrect symlinks")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Print the plan without writing files")
	return cmd
}
