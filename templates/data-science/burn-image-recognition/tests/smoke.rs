#[cfg(test)]
mod tests {
    use {{ project_name }}::model::Model;
    use burn_ndarray::NdArray;
    use burn::prelude::Backend;

    #[test]
    fn test_model_forward_shape() {
        let device = <NdArray as Backend>::Device::default();
        let model = Model::<NdArray>::new(&device);
        let input = burn::tensor::Tensor::<NdArray, 3>::zeros([1, 28, 28], &device);
        let output = model.forward(input);
        assert_eq!(output.dims()[0], 1);
        assert_eq!(output.dims()[1], 10); // num_classes
    }
}
