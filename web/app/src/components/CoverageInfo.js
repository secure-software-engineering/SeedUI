import Plot from 'react-plotly.js';

export default function CoverageInfo({ data, fuzzersInfo }) {
    var plot_data = [];
    Object.entries(data).forEach((fuzzer_configuration) => {
        const [fuzzer_configuration_id, entries] = fuzzer_configuration;
        var xvalues = [];
        var yvalues = [];
        var meta_data = [];

        // Get the keys, sort them, and access the values in sorted order
        const sortedKeys = Object.keys(entries).sort((a, b) => Number(a) - Number(b));
        for (const key of sortedKeys) {
            const value = entries[key];
            xvalues.push(Number(key));
            yvalues.push(value.fuzzer_coverage);
            meta_data.push(value.input_id);
        }
        
        plot_data.push({
            x: xvalues,
            y: yvalues,
            customdata: meta_data,
            hovertemplate: 'seed-%{customdata}<br >#edges: %{y}<extra></extra>',
            mode: 'lines+markers',
            type: 'scatter',
            name: `Run #${fuzzer_configuration_id}`,
            marker: {
                color: fuzzersInfo.get(Number(fuzzer_configuration_id)).color,
                size: 2,
            },
        });

    });

    var layout = {
        autosize: false,
        height: window.innerHeight / 3,
        width: window.innerWidth / 2.1,
        showlegend: true,
        uirevision:'true', // necessary for ui persistency https://plotly.com/javascript/uirevision/
        legend: {
            x: 0.5,  // Position in the middle
            y: 1.1,  // Position just above the plot area
            xanchor: 'center', // Center align horizontally
            yanchor: 'bottom',  // Align to the bottom of the legend
            orientation: 'h',    // Horizontal orientation
        },
        yaxis: { // first plot from bottom
            showline: true,
            title: {
                text: '# Bitmap edges',
                standoff: 10
            },
            zeroline: false,
            showspikes: true,
            spikemode: 'toaxis',
        },
        xaxis: {
            // tickformat: "HH:mm:ss",
            showline: true,
            type: 'date',
            tickformat: '%H:%M',
            ticks: 'inside',
            title: {
                text: 'Time (HH:mm)',
                standoff: 0
            },
            showspikes: true,
            spikemode: 'toaxis',
        },
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

    return (
        <>
            <Plot
                divId='coverage_graph'
                data={plot_data}
                layout={layout}
                config={{
                    displaylogo: false,
                    // displayModeBar: true,
                    // editable: true,
                    responsive: true,
                    modeBarButtonsToRemove: ['select2d', 'lasso2d', 'autoScale2d']
                }}
            />
        </>
    );
}