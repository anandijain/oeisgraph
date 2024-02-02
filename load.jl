using Graphs, CSV, DataFrames
using Catlab
using Catlab, Catlab.Theories
using Catlab.Theories
using Catlab.CategoricalAlgebra
using Catlab.Graphics

acode_to_int(x) = parse(Int, x[2:end])
acode_to_int.(vids)
draw(g, ) = to_graphviz(g, node_labels=true, edge_labels=true)

@acset_type IndexedLabeledGraph(Catlab.Graphs.SchLabeledGraph, index=[:src, :tgt],
    unique_index=[:label]) <: Catlab.Graphs.AbstractLabeledGraph



df = CSV.read("g.csv", DataFrame)
df = df[1:end-1, :]
df = df[.!(df.src .== df.dst), :]
# vids = String7.(unique(vcat(df[:, 1], df[:, 2])))
vids = String.(unique(vcat(df[:, 1], df[:, 2])))
NV = length(vids)
id_vid = Dict(vids .=> 1:NV)
vid_id = Dict(1:NV .=> vids)

g = IndexedLabeledGraph{String}()

Catlab.Graphs.add_vertices!(g, NV; label=vids)

for r in eachrow(df)
    Catlab.Graphs.add_edge!(g, id_vid[r.src], id_vid[r.dst])
end

dg = Graphs.DiGraph(g)
ccs = Graphs.connected_components(sdg)
ig = Catlab.Graphs.induced_subgraph(g, sort(ccs;by=length)[end-1])
# draw(ig)
to_graphviz(ig; node_labels=:label)

foo  =sort(degp; by=last, rev=true)
idk = map(x->vid_id[x], first.(foo))

ps = idk .=> last.(foo)

# print(map(x->vid_id[x], first(first.(ps), 20)))