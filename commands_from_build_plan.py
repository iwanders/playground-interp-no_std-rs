#!/usr/bin/env python3

import json
import sys

def load(path):
    with open(path) as f:
        return json.load(f)

def collect_args(invocation):
    args = []
    for arg in invocation["args"]:
        if (args and args[-1] == "--cfg"):
            arg = f"'{arg}'" # apply the odd quotes from https://github.com/rust-lang/rust/issues/66450#issue-523560575
        args.append(arg)
    return " ".join(args)

def print_commands(plan):
    for b in plan["invocations"]:
        print(f"{b['package_name']}")
        args = collect_args(b)
        print(f"  {b['program']} {args}\n")

if __name__ == "__main__":
    plan = load(sys.argv[1])
    print_commands(plan)
