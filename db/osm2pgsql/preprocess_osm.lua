local places = osm2pgsql.define_table({
	name = "places",
	ids = { type = "any", id_column = "osm_id" },
	columns = {
		{ column = "name", type = "text" },
		{ column = "feature_type", type = "text" },
		{ column = "geom", type = "geometry", projection = 4326 },
		{ column = "source", type = "text", not_null = true },
		{ column = "housenumber", type = "text" },
		{ column = "street", type = "text" },
		{ column = "city", type = "text" },
		{ column = "state", type = "text" },
		{ column = "country_code", type = "text" },
		{ column = "postcode", type = "text" },
	},
})

local FEATURE_TYPE_KEYS = {
	"place",
	"historic",
	"tourism",
	"leisure",
	"natural",
	"man_made",
	"amenity",
	"shop",
	"landuse",
	"building",
}

local FEATURE_TYPE_DENY = {
	information = true,
	locality = true,
	yes = true,
}

local function get_name(tags)
	return tags.name or tags["name:cs"] or tags["name:en"]
end

local function get_feature_type(tags)
	for _, key in ipairs(FEATURE_TYPE_KEYS) do
		local v = tags[key]
		if v and v ~= "" and not FEATURE_TYPE_DENY[v] then
			return v
		end
	end
	return nil
end

local function get_address(tags)
	return {
		housenumber = tags["addr:housenumber"],
		street = tags["addr:street"],
		city = tags["addr:city"],
		state = tags["addr:state"],
		country_code = tags["addr:country"],
		postcode = tags["addr:postcode"],
	}
end

function osm2pgsql.process_node(object)
	local name = get_name(object.tags)
	local ftype = get_feature_type(object.tags)
	if not name or not ftype then
		return
	end

	local addr = get_address(object.tags)
	places:insert({
		name = name,
		feature_type = ftype,
		geom = object:as_point(),
		source = "osm",
		housenumber = addr.housenumber,
		street = addr.street,
		city = addr.city,
		state = addr.state,
		country_code = addr.country_code,
		postcode = addr.postcode,
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

	local addr = get_address(object.tags)
	places:insert({
		name = name,
		feature_type = ftype,
		geom = object:as_polygon():centroid(),
		source = "osm",
		housenumber = addr.housenumber,
		street = addr.street,
		city = addr.city,
		state = addr.state,
		country_code = addr.country_code,
		postcode = addr.postcode,
	})
end
