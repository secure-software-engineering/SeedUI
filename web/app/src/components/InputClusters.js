import { Box, Stack, Typography } from '@mui/material';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select from '@mui/material/Select';
import { styled } from '@mui/system';
import Divider from '@mui/material/Divider';
import Chip from '@mui/material/Chip';
import OutlinedInput from '@mui/material/OutlinedInput';
import LinearProgress from '@mui/material/LinearProgress';

import { postInputClusters } from './fetchers.js'
import { useState } from 'react';
import Plot from 'react-plotly.js';

const SmallSelect = styled(Select)({
    height: '25px', // Adjust height
    fontSize: '0.8rem', // Adjust font size
    // padding: '0 5px', // Adjust padding
});


function InputClusters ({ fuzzersInfo }) {
    const [selectedClusterThreshold, setSelectedClusterThreshold] = useState(0);
    const [plotData, setPlotData] = useState([]);
    const [showGraph, setShowGraph] = useState(false);
    const [requestLoading, setRequestLoading] = useState(false);

    if (fuzzersInfo.size === 0) { return <LinearProgress sx={{ m: 5 }} />; }

    // console.log("fuzzers info: ", fuzzersInfo);

    const handleSelectedClusterThreshold = (event) => {
        setSelectedClusterThreshold(event.target.value);
    };

    const postClustertRequest = (clusterThreshold) => {
        setShowGraph(false);
        setRequestLoading(true);

        postInputClusters({
                        "cluster_threshold_seconds": clusterThreshold,
                    }).then(function(inputClusters) {
            // console.log("cluster data: ", inputClusters);
            const traces = new Map();
            const initial_seed_contributions = new Map();
            Object.entries(inputClusters).forEach((fuzzer_configuration) => {
                const [fuzzer_configuration_id, entries] = fuzzer_configuration;
                traces.set(fuzzer_configuration_id, new Map());
                traces.get(fuzzer_configuration_id).set('x', []);
                traces.get(fuzzer_configuration_id).set('y', []);
                initial_seed_contributions.set(fuzzer_configuration_id, new Map());
                initial_seed_contributions.get(fuzzer_configuration_id).set('x', []);
                initial_seed_contributions.get(fuzzer_configuration_id).set('y', []);
                initial_seed_contributions.get(fuzzer_configuration_id).set('bubble_size', []);
                initial_seed_contributions.get(fuzzer_configuration_id).set('initial_seeds_meta', []);
                
                Object.entries(entries).forEach((entry) => {
                    let [key, value] = entry;
                    Object.entries(value['inputs']).forEach((input_entry) => {
                        let [, line_bitmap_coverage_array] = input_entry;
                        traces.get(fuzzer_configuration_id).get('x').push(Number(key));
                        traces.get(fuzzer_configuration_id).get('y').push(line_bitmap_coverage_array['fuzzer_coverage']);
                    });

                    initial_seed_contributions.get(fuzzer_configuration_id).get('x').push(Number(key));
                    initial_seed_contributions.get(fuzzer_configuration_id).get('y').push(0);
                    initial_seed_contributions.get(fuzzer_configuration_id).get('bubble_size').push(Object.keys(value['initial_seeds']).length);
                    let initial_seeds_meta = `<i>Run #${fuzzer_configuration_id}<i><br>`;
                    Object.entries(value['initial_seeds']).forEach((initial_seed_entry) => {
                        const [input_id, is_fuzzer_coverage] = initial_seed_entry;
                        const contribution_percentage = (is_fuzzer_coverage / value['total_fuzzer_coverage']) * 100;
                        initial_seeds_meta += `%seed #${input_id} edges: <b>${contribution_percentage.toFixed(2)}%</b><br>`;
                    });
                    initial_seed_contributions.get(fuzzer_configuration_id).get('initial_seeds_meta').push(initial_seeds_meta);
                });

            });
            
            const plot_data = [];
            traces.forEach((trace_values, fuzzer_configuration_id) => {
                plot_data.push({
                    y: trace_values.get('y'),
                    x: trace_values.get('x'),
                    name: `Run #${fuzzer_configuration_id}`,
                    marker: {
                        color: fuzzersInfo.get(Number(fuzzer_configuration_id)).color,
                    },
                    yaxis: 'y',
                    type: 'box',
                });

            });

            initial_seed_contributions.forEach((seed_contributions, fuzzer_configuration_id) => {
                plot_data.push({
                    y: seed_contributions.get('y'),
                    x: seed_contributions.get('x'),
                    marker: {
                        size: seed_contributions.get('bubble_size').map(val => val * 7.5),
                        color: fuzzersInfo.get(Number(fuzzer_configuration_id)).color,
                        sizeref: 0.8,
                    },
                    text: seed_contributions.get('initial_seeds_meta'),
                    name: `Run #${fuzzer_configuration_id}`,
                    yaxis: 'y2',
                    type: 'scatter',
                    mode: 'markers',
                    // showlegend: false,
                    hoverinfo: 'text', // only show custom hover text
                    hovertemplate: `%{text}<extra></extra>`,
                });
            });

            setPlotData([...plot_data]);
            setRequestLoading(false);
            setShowGraph(true);
        });
    }

    const renderThresholdItems = () => {
        return [
            <MenuItem key="1 minutes" value={1} onClick={() => postClustertRequest(1)}>1 minute</MenuItem>, 
            <MenuItem key="2 minutes" value={2} onClick={() => postClustertRequest(2)}>2 minutes</MenuItem>, 
            <MenuItem key="5 minutes" value={5} onClick={() => postClustertRequest(5)}>5 minutes</MenuItem>, 
            <MenuItem key="10 minutes" value={10} onClick={() => postClustertRequest(10)}>10 minutes</MenuItem>,
            <MenuItem key="15 minutes" value={15} onClick={() => postClustertRequest(15)}>15 minutes</MenuItem>, 
            <MenuItem key="20 minutes" value={20} onClick={() => postClustertRequest(20)}>20 minutes</MenuItem>,
            <MenuItem key="25 minutes" value={25} onClick={() => postClustertRequest(25)}>25 minutes</MenuItem>,
            <MenuItem key="30 minutes" value={30} onClick={() => postClustertRequest(30)}>30 minutes</MenuItem>,
            <MenuItem key="45 minutes" value={45} onClick={() => postClustertRequest(45)}>45 minutes</MenuItem>,
            <MenuItem key="60 minutes" value={60} onClick={() => postClustertRequest(60)}>60 minutes</MenuItem>,
        ];
    };


    const layout = {
        grid: {
            rows: 2, 
            columns: 1, 
            subplots:[['xy2'], ['xy']],
        },
        autosize: false,
        height: window.innerHeight / 3,
        width: window.innerWidth / 2.1,
        showlegend: true,
        uirevision:'true', // necessary for ui persistency https://plotly.com/javascript/uirevision/
        legend: {
            x: 0.5,  // Position in the middle
            y: 1,  // Position just above the plot area
            xanchor: 'center', // Center align horizontally
            yanchor: 'bottom',  // Align to the bottom of the legend
            orientation: 'h'    // Horizontal orientation
        },
        yaxis: {
            domain: [0, 0.75],
            showline: true,
            title: {
                text: '# Bitmap edges',
                standoff: 10,
            },
            zeroline: false
        },
        yaxis2: {
            domain: [0.75, 1],
            visible: false,
            showline: true,
            zeroline: false,
            legend: false,
        },
        xaxis: {
            showline: true,
            type: 'date',
            tickformat: '%H:%M',
            ticks: 'inside',
            title: {
                text: 'Time (HH:mm)',
                standoff: 1
            },
        },
        boxmode: 'group',
        scattermode: 'group',
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
            <Divider sx={{ mt: 0.1, mb: 0.1, textAlign: 'center' }} component="div" role="presentation" variant='fullWidth'>
                <Stack direction="row" justifyContent={'center'} spacing={2}>
                    <Chip sx={{ m: 0, p: 0, textAlign: 'center' }} label="Seed Clusters" size='small' variant='filled' color="success" />
                    <FormControl variant="left" sx={{ m: 0.1, mb: 0.2, minWidth: 120 }}>
                        <SmallSelect
                            displayEmpty
                            input={<OutlinedInput />}
                            labelId="threshold-label"
                            id="select-threshold-label-id"
                            value={`${selectedClusterThreshold} minute(s)`}
                            onChange={handleSelectedClusterThreshold}
                            renderValue={(selected) => {
                                if (selected.length === 0) {
                                    return <em>Placeholder</em>;
                                }

                                return selected;
                            }}
                        >
                        <MenuItem disabled value="">
                            <em>Threshold</em>
                        </MenuItem>
                        {
                            renderThresholdItems()
                        }
                        </SmallSelect>
                    </FormControl>
                </Stack>
            </Divider>
            <Box sx={{ display: 'flex', flexDirection: 'column', overflow: 'auto', height: window.innerHeight / 3 }} >
                { showGraph && (
                    <Plot
                        data={plotData}
                        layout={layout}
                        config={{
                            displaylogo: false,
                            // displayModeBar: true,
                            // editable: true,
                            responsive: true,
                            modeBarButtonsToRemove: ['select2d', 'lasso2d', 'autoScale2d']
                        }}
                    />
                    )
                }
                { !showGraph && selectedClusterThreshold === 0 && (
                    <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 1 }}>
                        <Typography>No threshold is provided.</Typography>
                    </Box>
                    )
                }
                {
                    !showGraph && selectedClusterThreshold !== 0 && requestLoading && <LinearProgress sx={{ m: 5 }} />
                }
            </Box>
        </>
    
}

export default InputClusters;