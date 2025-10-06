import { Box, Stack, Typography } from '@mui/material';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select from '@mui/material/Select';
import { styled } from '@mui/system';
import Divider from '@mui/material/Divider';
import Chip from '@mui/material/Chip';
import OutlinedInput from '@mui/material/OutlinedInput';
import LinearProgress from '@mui/material/LinearProgress';

import { useState } from 'react';
import { postInitialSeedTimeline } from './fetchers.js'

import Plot from 'react-plotly.js';

const SmallSelect = styled(Select)({
    height: '25px', // Adjust height
    fontSize: '0.8rem', // Adjust font size
    // padding: '0 5px', // Adjust padding
});


export default function InputsTimeline({ fuzzersInfo }) {

    const [selectedFuzzerConfiguration, setSelectedFuzzerConfiguration] = useState(-1);
    const [selectedInitialSeedId, setSelectedInitialSeedId] = useState([]);
    const [plotData, setPlotData] = useState([]);
    const [showGraph, setShowGraph] = useState(false);
    const [requestLoading, setRequestLoading] = useState(false);

    if (fuzzersInfo.size === 0) { return <LinearProgress sx={{ m: 5 }} />; }
    
    const handleFuzzerConfigurationChange = (event) => {
        setSelectedFuzzerConfiguration(event.target.value);
        setSelectedInitialSeedId([]);
    };

    const handleInitialSeedChange = (event) => {
        if (Array.from(event.target.value).length > 0) {
            setSelectedInitialSeedId(event.target.value);
            renderTimeline(event.target.value);
        } else {
            setSelectedInitialSeedId([]);
            setShowGraph(false);
        }
    };

    const renderTimeline = (selectedIS) => {
        if (selectedFuzzerConfiguration !== -1 && Number(selectedIS) !== -1) {
            setShowGraph(false);
            setRequestLoading(true);
            const selectedISs = selectedIS.map(Number);
            postInitialSeedTimeline({
                "fuzzer_configuration_id": selectedFuzzerConfiguration,
                "initial_seed_ids": selectedISs,
            }).then(function(seedTimeline) {
                // console.log("timeline: ", seedTimeline);
                const nodesData = seedTimeline.nodes;
                const linksData = seedTimeline.edges;
                const edgeLines = [];
                    linksData.forEach(link => {
                        const sourceNode = nodesData.find(node => node.id === link.source);
                        const targetNode = nodesData.find(node => node.id === link.target);
                        // NOTE: there can be empty source node when the child is derived from grand child of another initial seed
                        if (sourceNode && targetNode) {
                            edgeLines.push({
                                x: [sourceNode.x_executed_on, targetNode.x_executed_on],
                                y: [sourceNode.y_fuzzer_coverage, targetNode.y_fuzzer_coverage],
                                mode: 'lines',
                                line: { color: 'gray' },
                                name: '',
                            });
                        }
                    });
                
                // Identify grandchildren with multiple parents
                const grandchildIds = linksData.reduce((acc, link) => {
                    if (!acc[link.target]) {
                        acc[link.target] = 0;
                    }
                    acc[link.target]++;
                    return acc;
                }, {});

                // Prepare nodes with different colors
                const nodeColors = nodesData.map(node => {
                    // Check if the node is a grandchild with multiple parents
                    return grandchildIds[node.id] > 1 ? 'red' : fuzzersInfo.get(Number(selectedFuzzerConfiguration)).color; // Red for grandchildren, blue for others
                });

                const nodePositions = {
                    x: nodesData.map(node => node.x_executed_on),
                    y: nodesData.map(node => node.y_fuzzer_coverage),
                    text: nodesData.map(node => `${node.name}<br>from: [${node.meta_data}]`),
                    mode: 'markers',
                    marker: {
                        color: nodeColors
                    },
                    type: 'scatter',
                    hoverinfo: 'text', // only show custom hover text
                    hovertemplate: `%{text}<extra></extra>`,
                };

                // Combine all traces
                setPlotData([...edgeLines, nodePositions]);
                setRequestLoading(false);
                setShowGraph(true);
            });
        }
    }

    const renderFuzzerConfigurationMenuItems = () => {
      return Array.from(fuzzersInfo.entries()).map(([key, value]) => (
            <MenuItem value={value.fuzzer_configuration_id}>{value.fuzzer_configuration_id}</MenuItem>
        ));
    };

    const renderInitialSeeds = () => {
        if (selectedFuzzerConfiguration === -1) {
            return <MenuItem value={-1}></MenuItem>;
        }

        const menuItems = [];
        fuzzersInfo.forEach((value, index) => {
            if (value.fuzzer_configuration_id === selectedFuzzerConfiguration) {
                Object.entries(value.initial_seeds_children_input_id_map).forEach((initial_seed_children_map, index_inner) => {
                    menuItems.push(<MenuItem key={`timeline-seed-${value.fuzzer_configuration_id}-${initial_seed_children_map[0]}`} value={initial_seed_children_map[0]} >{`seed #${initial_seed_children_map[0]}`}</MenuItem>);
                });
            }
        });

        return menuItems;
    };

    const layout = {
        autosize: false,
        height: window.innerHeight / 3.25,
        width: window.innerWidth / 2.125,
        uirevision:'true', // necessary for ui persistency https://plotly.com/javascript/uirevision/
        xaxis: {
            showline: true,
            type: 'date',
            tickformat: '%H:%M',
            ticks: 'inside',
            title: {
                text: 'Time (HH:mm)',
                standoff: 1
            },
            showspikes: true,
            spikemode: 'toaxis',
            showgrid: true,
        },
        yaxis: {
            showgrid: true,
            showspikes: true,
            spikemode: 'toaxis',
            showline: true,
            title: {
                text: '# Bitmap edges',
                standoff: 10,
            },
            zeroline: false
        },
        showlegend: false,
        margin: {
            r: 5,
            t: 2,
            b: 45,
            pad: 0
	    },
        modebar: {
            orientation: 'v',
        },
    };

    return <>
        <Divider sx={{ mt: 0, mb: 1}} component="div" role="presentation" variant='fullWidth' textAlign="center" >
            <Stack direction="row" justifyContent={'center'} spacing={2}>
                <Chip sx={{ m: 0, p: 0 }} label="Seed Timeline" size='small' variant='filled' color="success" />
                <FormControl variant="standard" sx={{ m: 0.1, mb: 0.2, minWidth: 80 }}>
                    <SmallSelect
                        displayEmpty
                        input={<OutlinedInput />}
                        labelId="configuration-label"
                        id="select-configuration-label-id"
                        value={selectedFuzzerConfiguration === -1 ? `Run #` : `Run #${selectedFuzzerConfiguration}`}
                        onChange={handleFuzzerConfigurationChange}
                        renderValue={(selected) => {
                            if (selected.length === 0) {
                                return <em>Configuration</em>;
                            }

                            return selected;
                        }}
                    >
                    <MenuItem disabled value="">
                        <em>Configuration</em>
                    </MenuItem>
                    {
                        renderFuzzerConfigurationMenuItems()
                    }
                    </SmallSelect>
                </FormControl>
                <FormControl variant="standard" sx={{ mt: 0.1, ml: 1, mb: 0.2, minWidth: 80 }}>
                    <SmallSelect
                        displayEmpty
                        multiple
                        input={<OutlinedInput />}
                        labelId="initial-seed-label"
                        id="select-initial-seed-label-id"
                        value={Array.from(selectedInitialSeedId)}
                        onChange={handleInitialSeedChange}
                        disabled={selectedFuzzerConfiguration === -1}
                    >
                    <MenuItem disabled value={[""]}>
                        <em>Initial seed</em>
                    </MenuItem>
                    {
                        renderInitialSeeds()
                    }
                    </SmallSelect>
                </FormControl>
            </Stack>
        </Divider>
        <Box sx={{ display: 'flex', flexDirection: 'column', overflow: 'auto', height: window.innerHeight / 3.2 }} >
            { showGraph && (
                <Plot
                    data={plotData}
                    layout={layout}
                    config={{
                        displaylogo: false,
                        // editable: true,
                        responsive: true,
                        modeBarButtonsToRemove: ['select2d', 'lasso2d', 'autoScale2d']
                    }}
                />
                )
            }
            { !showGraph && Array.from(selectedInitialSeedId).length === 0 && (
                <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 1 }}>
                    <Typography>No initial seed is selected to display its mutation timeline.</Typography>
                </Box>
                )
            }
            {
                !showGraph && Array.from(selectedInitialSeedId).length > 0 && requestLoading && <LinearProgress sx={{ m: 5 }} />
            }
        </Box>
    </>
}