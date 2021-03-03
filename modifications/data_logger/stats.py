from collections import defaultdict

class FunctionData:

    def __init__(self, name):
        self.name = name
        self.call_count = 1
        self.calls = defaultdict(lambda: 0)
    


if __name__ == "__main__":
    functions = {}
    
    with open("log") as f:
        lines = [line for line in f]

        for line in lines[1:]:
            name, est, act = line.split(",")

            if name == "run_application":
                continue

            if name not in functions:
                functions[name] = FunctionData(name)
            else:
                functions[name].call_count += 1

    with open("calls") as f:
        lines = [line for line in f]

        for line in lines[1:]:
            name, caller, callee = line.strip().split(",")

            if caller == "entry_point":
                continue

            functions[caller].calls[callee] += 1


    for f in functions.values():
        print(f"f_name: {f.name}, call_count: {f.call_count}")

        for k,v in f.calls.items():
            print(f"  {k} {round(v/f.call_count, 2)}")
