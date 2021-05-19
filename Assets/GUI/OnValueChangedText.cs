using UnityEngine;
using UnityEngine.UI;

public class OnValueChangedText : MonoBehaviour
{

    Slider slider;
    public string label;

    public Text attachedValue;
    public void Awake()
    {
        slider = GetComponent<Slider>();
        slider.onValueChanged.AddListener(delegate { OnSliderWasChanged(); });
        OnSliderWasChanged();
    }

    // OnDisabled() & OnDestroy(): When the GameObject is not in use anymore, we
    // should use "slider.onValueChanged.RemoveAllListeners ();". This removes any
    // Listeners we added via code; such as our "OnSliderWasChanged" method.
    void OnDisabled() { slider.onValueChanged.RemoveAllListeners(); }
    void OnDestroy() { slider.onValueChanged.RemoveAllListeners(); }

    // These are methods to change the value of the slider programatically - this is 
    // more useful for sliders you have disabled.
    public void ChangeSlider(float h) { slider.value = h; }
    public void incrementSlider(float h) { slider.value += h; }
    public void decrementSlider(float h) { slider.value -= h; }

    public void OnSliderWasChanged()
    {
        attachedValue.text = label + " " + slider.value.ToString("0.00");
    }
}