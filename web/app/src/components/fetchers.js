import useSWR from "swr";
import useSWRImmtable from "swr/immutable";
import axios from 'axios';

const instance = axios.create({
    baseURL: "http://127.0.0.1:8080/",
    timeout: 40000,
    headers: {
        Accept: 'application/json, text/plain, */*',
        'Content-Type': 'application/json; charset=utf-8'
    }
});


const get_fetcher = (url) => {
    return instance.get(url).then((res) => {
        if (!res.data) {
            throw Error(res.data.message);
        }

        return res.data;
    });
};

function getFuzzerInfo() {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const { data, error, isLoading } = useSWRImmtable("/fuzzer_info", get_fetcher);

    return {
        data,
        isLoading,
        error
    }
}

function postLineCoverageOvertime(post_data, setData) {
    instance.post("/line_coverage", post_data)
        .then(response => setData(response.data));
}

async function postCompareInputs(post_data, setData) {
    return await instance.post("/compare_inputs", post_data)
        .then(response => response.data);
}

function postInitialSeedsCoverage(post_data, setData) {
    instance.post("/initial_seeds_line_coverage_for_file", post_data)
        .then(response => setData(response.data));
}

async function postInitialSeedsChildCoverage(post_data) {
    return await instance.post("/line_coverage_for_file", post_data)
        .then(response => response.data);
}

async function postInitialSeedTimeline(post_data) {
    return await instance.post("/initial_seed_timeline", post_data)
        .then(response => response.data);
}

async function postInputClusters(post_data) {
    return await instance.post("/input_clusters", post_data)
        .then(response => response.data);
}

function getSUT() {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const { data, error, isLoading } = useSWR("/sut", get_fetcher);

    return {
        data,
        isLoading,
        error
    }
}

function getSUTFileInfoMap() {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const { data, error, isLoading } = useSWR("/sut_file_info", get_fetcher);

    return {
        data,
        isLoading,
        error
    }
}

export { postLineCoverageOvertime, postCompareInputs, postInitialSeedsCoverage, postInitialSeedTimeline, postInitialSeedsChildCoverage, getSUT, getSUTFileInfoMap, getFuzzerInfo, postInputClusters };