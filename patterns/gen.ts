#!/usr/bin/env -S deno run

const { args } = Deno;

if (args[0] == "--help") {
	console.log("usage: [bin] <size> <frequency>");
}

const size = parseInt(args[0] ?? "5");
const frequency = parseFloat(args[1] ?? "0.5");


function* range(from: number, to: number) {
    let current = from;
    while (current < to) yield current++;
}

let result = "";
for (const _y of range(0, size)) {
    for (const _x of range(0, size))
        result += Math.random() < frequency ? '#' : ' ';
    result += '\n';
}

console.log(result);
