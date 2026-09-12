// Domain adapter for HyCC 839763c1a9ec286d0cf7341886ff15f3b281d266.
// Uses the pinned upstream reader and ordering; does not implement a crypto core.
#include <circuit-utils/circuit_io.hpp>
#include <iostream>
#include <iomanip>

static void endpoint(circ::WireEndpoint e) {
    std::cout << '[' << e.id.raw() << ',' << unsigned(e.pin) << ']';
}
static void name_hex(std::string const& name) {
    std::cout << '"';
    for (unsigned char c : name)
        std::cout << std::hex << std::setw(2) << std::setfill('0') << unsigned(c);
    std::cout << std::dec << '"';
}
int main(int argc, char** argv) try {
    if (argc != 2) throw std::runtime_error("expected one merged .circ path");
    // Upstream reader prints statistics; keep the machine stream JSON-only.
    auto* saved = std::cout.rdbuf(std::cerr.rdbuf());
    auto c = circ::read_circuit(argv[1], circ::CircuitFileFormat::cbmc_gc);
    std::cout.rdbuf(saved);
    if (!c.function_calls.empty()) throw std::runtime_error("unmerged external calls");
    std::cout << "{\"version\":1,\"inputs\":[";
    for (size_t i = 0; i < c.inputs.size(); ++i) {
        if (i) std::cout << ',';
        std::cout << '[' << circ::ElementID{circ::InputID{i}}.raw() << ','
                  << unsigned(c.inputs[i].width) << ']';
    }
    std::cout << "],\"gates\":[";
    bool first = true;
    circ::topological_traversal(c, [&](circ::ElementID id) {
        if (id.kind() != circ::ElementID::Kind::gate) return;
        auto const& g = c[id.as_gate_id()];
        if (!first) std::cout << ',';
        first = false;
        std::cout << "{\"id\":" << id.raw() << ",\"kind\":\"" << circ::to_string(g.kind)
                  << "\",\"width\":" << unsigned(g.width) << ",\"value\":" << g.const_value
                  << ",\"inputs\":[";
        for (size_t i = 0; i < g.fanins.size(); ++i) {
            if (i) std::cout << ',';
            endpoint(g.fanins[i]);
        }
        std::cout << "]}";
    });
    std::cout << "],\"outputs\":[";
    for (size_t i = 0; i < c.outputs.size(); ++i) {
        if (i) std::cout << ',';
        endpoint(c.outputs[i].value());
    }
    std::cout << "],\"input_groups\":[";
    first = true;
    for (auto p : c.ordered_inputs) {
        if (!first) std::cout << ',';
        first = false;
        std::cout << "{\"name_hex\":"; name_hex(p->name);
        std::cout << ",\"indices\":[";
        for (size_t i = 0; i < p->inputs.size(); ++i) {
            if (i) std::cout << ',';
            std::cout << p->inputs[i].value;
        }
        std::cout << "]}";
    }
    std::cout << "],\"output_groups\":[";
    first = true;
    for (auto p : c.ordered_outputs) {
        if (!first) std::cout << ',';
        first = false;
        std::cout << "{\"name_hex\":"; name_hex(p->name);
        std::cout << ",\"indices\":[";
        for (size_t i = 0; i < p->outputs.size(); ++i) {
            if (i) std::cout << ',';
            std::cout << p->outputs[i].value;
        }
        std::cout << "]}";
    }
    std::cout << "]}\n";
    if (!std::cout) throw std::runtime_error("output failure");
    return 0;
} catch (std::exception const& e) {
    std::cerr << e.what() << '\n';
    return 1;
}
