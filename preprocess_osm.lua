local places = osm2pgsql.define_table({
	name = "places",
	ids = { type = "any", id_column = "osm_id" },
	columns = {
		{ column = "name", type = "text" },
		{ column = "feature_type", type = "text" },
		{ column = "geom", type = "geometry", projection = 4326 },
		{ column = "source", type = "text", not_null = true },
	},
})

local function get_name(tags)
	return tags.name or tags["name:cs"] or tags["name:en"]
end

local function get_feature_type(tags)
	return tags.place or tags.amenity or tags.shop or tags.landuse or tags.building
end

function osm2pgsql.process_node(object)
	local name = get_name(object.tags)
	local ftype = get_feature_type(object.tags)
	if not name or not ftype then
		return
	end

	places:insert({
		name = name,
		feature_type = ftype,
		geom = object:as_point(),
		source = "osm",
	})
end

function osm2pgsql.process_way(object)
	if not object.is_closed then
		return
	end

	local name = get_name(object.tags)
	local ftype = get_feature_type(object.tags)
	if not name or not ftype then
		return
	end

	places:insert({
		name = name,
		feature_type = ftype,
		geom = object:as_polygon():centroid(),
		source = "osm",
	})
end
