$(document).ready(function () {
    var pinned = false;
    $(".treemap-view").hide()

    $('.account-nav').click(function (e) {
        e.preventDefault();
        $('.arrow').toggleClass('rotate90');
        $('.droplist').toggle()
    });

    $('.phone-menu').click(function (e) {
        e.preventDefault();
        // $('.arrow').toggleClass('rotate90');
        $('.menu-phone-hidden').toggle()
    });

    $(".graph-view-btn, .cls-btn").click(function (e) {
        e.preventDefault();

        if ($('#aside').is(':hidden') && $('#aside').hasClass('ray-grid__cell--span-12-desktop')) {
            $('.test').toggle()
            $('#aside').toggle()
            $('.main').hide()
            $('body').addClass("overflow-hide")

        } else if ($('#aside').is(':hidden') && pinned && $('.main').hasClass('ray-grid__cell--span-12-desktop')) {
            $(".main").removeClass("ray-grid__cell--span-12-desktop")
            $(".main").addClass("ray-grid__cell--span-5-desktop")
            $(".main").removeClass("abs")

            $('.test').toggle()
            $('#aside').toggle()
            $('body').removeClass("overflow-hide")

        } else {
            $('.test').toggle()
            $('#aside').toggle()
            $('.main').show()
            $('body').toggleClass("overflow-hide")

            if ($('.main').hasClass('ray-grid__cell--span-5-desktop')) {
                $(".main").removeClass("ray-grid__cell--span-5-desktop")
                $(".main").addClass("ray-grid__cell--span-12-desktop")
                $('body').removeClass("overflow-hide")
                $(".main").addClass("abs")
            }
        }

    })

    $(".pin-btn").click(function (e) {
        e.preventDefault()

        pinned = !pinned;
        $(".main").toggleClass("abs")
        $("body").toggleClass("overflow-hide")


        if ($('.main').hasClass('ray-grid__cell--span-12-desktop')) {
            $(".main").removeClass("ray-grid__cell--span-12-desktop")
            $(".main").addClass("ray-grid__cell--span-5-desktop")
        } else {
            $(".main").addClass("ray-grid__cell--span-12-desktop")
            $(".main").removeClass("ray-grid__cell--span-5-desktop")
        }

    })

    $('.wde-btn').click(function (e) {
        e.preventDefault()

        $('.test').removeClass("ray-grid__cell--span-7-desktop")
        $('.test').addClass("ray-grid__cell--span-12-desktop")
        $('.pin-btn').attr("disabled", true)
        $('main').hide()

    })

    $('.dft-btn').click(function (e) {
        e.preventDefault()

        $('.test').removeClass("ray-grid__cell--span-12-desktop")
        $('.test').addClass("ray-grid__cell--span-7-desktop")
        $('.pin-btn').attr("disabled", false)
        $('main').show()

    })

    $('.fd-btn').click(function (e) {
        e.preventDefault()

        $(".fd-view").show()
        $(".treemap-view").hide()

    })

    $('.treemap-btn').click(function (e) {
        e.preventDefault()

        $(".fd-view").hide()
        $(".treemap-view").show()

    })

    $('#home-logo-btn').click(function(e){
        e.preventDefault()

        $('#dash-container').show();
        $('#search-results').hide();
    })



    // $('.form-control').keypress(function (e) {
    //     if (e.which == 13) {
           
    //     }
    // });

    $(document).click(
        function (e) {
            var target = e.target;
            if (target.closest && $('.droplist').is(":visible")) {
                if (!(target.closest(".account-nav") || target.closest(".droplist"))) {

                    $('.arrow').toggleClass('rotate90');
                    $('.droplist').toggle()
                }
            }

            if (target.closest && $('.phone-menu').is(":visible")) {
                if (!(target.closest(".account-nav") || target.closest(".droplist") || target.closest(".phone-menu") || target.closest(".search"))) {

                    if ($('.menu-phone-hidden').is(":visible")) {
                        $('.menu-phone-hidden').toggle()
                    }
                }
            }
        }
    );
});